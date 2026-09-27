//! E161（崩溃放量的去重与分段耗时）跑前登记第一段：入库装置。
//!
// admission: always 每跑一次都在今天这份 crates 的恢复与 checker 上重量一遍逐状态的性质与分段耗时，计时每次都是新观测
// run-condition: command python3
//!
//! 登记：`research/prompts/e161-preregistration.md`（5.5「第一段」：D1-small、D1-stride、D1-quick，停机条款 S1–S6、
//! 锚点 A1–A7、5.3 全部阳性对照、第八节几何敏感性里 D1-quick 与线程数 1 那两格）。第二、三段（D2、D3 全量）是同一个装置换取样域，
//! 本文件给了模式名但这一段不跑。
//!
//! 只读、只驱动、只观测：不改 `crates/singlefs-core`、`crates/singlefs-checker`、harness 已有的文件。
//! 第一条流经 `#[path]` 引 `tests/common/mod.rs` 的 `build_pool`；第二条流的脚本照抄
//! `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:203-474`（到 E 再正常卸载那一支）。
//! 决定几何与门槛的数都写成本地常量（值抄自登记第十一节 S1 与第七节），`main` 开头逐条与 crates 的同名常量回比（S1）。
//! 枚举计划与撕裂镜像表是本文件自己的一份，与 crates 私有的那一份逐状态对拍（S3）。
//!
//!   e161_crash_state_dedup_and_time_split segment-one
//!
//! 线程数取环境变量 `E161_THREADS`，没设取 `available_parallelism`。产物是 `E7RESULT` 行，收尾 `name=done emitted=N`。
//! 退出码：0 正常；2 命令行不对；3 停机条款没过（S1–S5，产物照打、交主 agent）；preflight 拒绝时照它的退出码退。

#[path = "../../tests/common/mod.rs"]
mod common;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use singlefs_checker::image::{
    chosen_system_configurations, valid_roots, ImageReader, InvariantVerdict,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::recovery::{
    recover, JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
};
use singlefs_harness::crash::{
    check_records, newest_persisted_root, CrashImage, Layer0SegmentExpansion, Layer0Tally,
    MemoryPool, PublishedVersion, RetainedWrite, WrittenContents,
};
use singlefs_harness::segments::StepKind;

// ───────────────────────────── 本地常量（S1 回比） ─────────────────────────────

/// 16 KiB 槽（登记 S1：`singlefs_format::SLOT_BYTES` 16384）。
const LOCAL_SLOT_BYTES: u64 = 16_384;
/// 索引节点单元长（S1：`NODE_BYTES` 16384）。
const LOCAL_NODE_BYTES: u64 = 16_384;
/// 数据单元与打包单元长（S1：`DATA_UNIT_BYTES` 32768）。
const LOCAL_DATA_UNIT_BYTES: u64 = 32_768;
/// 一条 journal 记录（S1：`JOURNAL_RECORD_BYTES` 4096）。
const LOCAL_JOURNAL_RECORD_BYTES: u64 = 4_096;
/// journal 环起点槽（S1：`JOURNAL_RING_START_SLOT` 1024）。
const LOCAL_JOURNAL_RING_START_SLOT: u64 = 1_024;
/// journal 环默认字节数（S1：`JOURNAL_RING_DEFAULT_BYTES` 805306368）。
const LOCAL_JOURNAL_RING_DEFAULT_BYTES: u64 = 805_306_368;
/// 单元区起点槽（S1：`UNIT_AREA_START_SLOT` 50176）。
const LOCAL_UNIT_AREA_START_SLOT: u64 = 50_176;
/// 层 0 扇区（S1：`singlefs_harness::crash::SECTOR_BYTES` 512）。
const LOCAL_SECTOR_BYTES: u64 = 512;
/// 五种 key 宽（S1：extent 24、inode 8、分配 10、记账 22、映射 27）与树表 8。
const LOCAL_KEY_WIDTH_EXTENT: usize = 24;
const LOCAL_KEY_WIDTH_INODE: usize = 8;
const LOCAL_KEY_WIDTH_ALLOCATION: usize = 10;
const LOCAL_KEY_WIDTH_ACCOUNTING: usize = 22;
const LOCAL_KEY_WIDTH_MAPPING: usize = 27;
const LOCAL_KEY_WIDTH_TREE_TABLE: usize = 8;

/// journal 环在盘上的起止（字节）：起点 1024 × 16384，终点 = 单元区起点 50176 × 16384。
const RING_START_BYTES: u64 = LOCAL_JOURNAL_RING_START_SLOT * LOCAL_SLOT_BYTES;
const UNIT_AREA_START_BYTES: u64 = LOCAL_UNIT_AREA_START_SLOT * LOCAL_SLOT_BYTES;

/// 登记第一节「接近 1 / 接近状态数」与 G3「很小」的门槛：0.5。
const RATIO_THRESHOLD_NUMERATOR: u64 = 1;
const RATIO_THRESHOLD_DENOMINATOR: u64 = 2;
/// 第一段 G3 够判：s_lo、s_hi 在 0.5 同一侧且离 0.5 至少 0.1（5.5）。
const SHARE_MARGIN_FOR_ENOUGH: f64 = 0.1;
/// D1-stride 的步长（5.5：素数 2311）。
const STRIDE_STEP: u64 = 2_311;
/// 对照样本里大段各取序号最小的这么多个（5.5）。
const CONTROL_SAMPLE_PER_LARGE_SEGMENT: usize = 4_096;
/// PC3：第二条流段 14 里序号最小的 1000 个状态、每个计时区各加 2 ms 忙等（5.3）。
const TIMING_CONTROL_STATES: usize = 1_000;
const TIMING_CONTROL_BUSY_WAIT: Duration = Duration::from_millis(2);
/// PC3 的判据：加忙等的那个区多出 ≥ 1.8 s，其余每个区变化 < 1 s。
const TIMING_CONTROL_MINIMUM_ADDED_NANOSECONDS: u128 = 1_800_000_000;
const TIMING_CONTROL_MAXIMUM_OTHER_CHANGE_NANOSECONDS: u128 = 1_000_000_000;
/// 容量（A6）：单卡最大 32607 MiB、16 GB 那一档里最小 16303 MiB、五卡合计 97843 MiB。
const MEBIBYTE: u128 = 1 << 20;
const LARGEST_CARD_MEBIBYTES: u128 = 32_607;
const SMALLEST_SIXTEEN_GIGABYTE_CARD_MEBIBYTES: u128 = 16_303;
const FIVE_CARDS_MEBIBYTES: u128 = 97_843;
const CARD_COUNT: u128 = 5;
/// 外推点：问题单的 10¹⁰，另报第二条流整条流的全量状态数（第四节 4.2）。
const EXTRAPOLATED_STATES: u128 = 10_000_000_000;
const SECOND_STREAM_FULL_STATES: u128 = 14_960_689_284;
/// A7：第二条流整条流最大一段 3² · 2²⁸ − 1。
const SECOND_STREAM_LARGEST_SEGMENT_STATES: u128 = 2_415_919_103;
/// 容量的 95% 到 100% 之间记「临界」（第六节 G5 表下）。
const CRITICAL_BAND_PERCENT: u128 = 95;
/// G5 的字节口径（第六节）：读集每次调用 21 字节，集合每个元素 20 字节，每状态三个键各 16 字节，每个不同键一条 20 字节。
const READ_SET_BYTES_PER_CALL: u64 = 21;
const SET_BYTES_PER_ELEMENT: u64 = 20;
const KEY_BYTES: u64 = 16;
const KEYS_STORED_PER_STATE: u64 = 3;
const DISTINCT_KEY_ENTRY_BYTES: u64 = 20;
/// 轨迹检查点（8.1）：段内每 2²⁰ 个状态一个；D1-stride 按取样状态每 2¹² 个一个。
const TRAJECTORY_STATES_PER_CHECKPOINT: u64 = 1 << 20;
const TRAJECTORY_STRIDE_SAMPLES_PER_CHECKPOINT: u64 = 1 << 12;
/// 一片几个状态：工作线程按片领活。
const STATES_PER_SLICE: usize = 256;
/// 线程数的环境变量（登记 5.6）。
const THREADS_ENVIRONMENT_VARIABLE: &str = "E161_THREADS";

/// S1：本地常量与 crates 同名常量逐条回比；交回（名字, 本地值, crates 值）。
fn local_constants_against_the_crates() -> Vec<(&'static str, u64, u64)> {
    let width =
        |schema: singlefs_checker::KeySchema| u64::try_from(schema.width()).expect("key 宽");
    let local = |value: usize| u64::try_from(value).expect("key 宽");
    vec![
        ("slot_bytes", LOCAL_SLOT_BYTES, singlefs_format::SLOT_BYTES),
        ("node_bytes", LOCAL_NODE_BYTES, singlefs_format::NODE_BYTES),
        (
            "data_unit_bytes",
            LOCAL_DATA_UNIT_BYTES,
            singlefs_format::DATA_UNIT_BYTES,
        ),
        (
            "journal_record_bytes",
            LOCAL_JOURNAL_RECORD_BYTES,
            singlefs_format::JOURNAL_RECORD_BYTES,
        ),
        (
            "journal_ring_start_slot",
            LOCAL_JOURNAL_RING_START_SLOT,
            singlefs_format::JOURNAL_RING_START_SLOT,
        ),
        (
            "journal_ring_default_bytes",
            LOCAL_JOURNAL_RING_DEFAULT_BYTES,
            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        ),
        (
            "unit_area_start_slot",
            LOCAL_UNIT_AREA_START_SLOT,
            singlefs_format::UNIT_AREA_START_SLOT,
        ),
        (
            "sector_bytes",
            LOCAL_SECTOR_BYTES,
            singlefs_harness::crash::SECTOR_BYTES,
        ),
        (
            "key_width_extent",
            local(LOCAL_KEY_WIDTH_EXTENT),
            width(singlefs_checker::KEY_SCHEMA_EXTENT),
        ),
        (
            "key_width_inode",
            local(LOCAL_KEY_WIDTH_INODE),
            width(singlefs_checker::KEY_SCHEMA_INODE),
        ),
        (
            "key_width_allocation",
            local(LOCAL_KEY_WIDTH_ALLOCATION),
            width(singlefs_checker::KEY_SCHEMA_ALLOCATION),
        ),
        (
            "key_width_accounting",
            local(LOCAL_KEY_WIDTH_ACCOUNTING),
            width(singlefs_checker::KEY_SCHEMA_ACCOUNTING),
        ),
        (
            "key_width_mapping",
            local(LOCAL_KEY_WIDTH_MAPPING),
            width(singlefs_checker::KEY_SCHEMA_MAPPING),
        ),
        (
            "key_width_tree_table",
            local(LOCAL_KEY_WIDTH_TREE_TABLE),
            width(singlefs_checker::KEY_SCHEMA_TREE_TABLE),
        ),
    ]
}

// ───────────────────────────── 128 位确定性散列（5.1） ─────────────────────────────

/// 128 位摘要：内容与键都用它。确定性（与线程数、跑的次数无关），std 之外自己写。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Digest128(u128);

const DIGEST_MULTIPLIER_LOW: u64 = 0x9E37_79B9_7F4A_7C15;
const DIGEST_MULTIPLIER_HIGH: u64 = 0xC2B2_AE3D_27D4_EB4F;
const DIGEST_SEED_LOW: u64 = 0x243F_6A88_85A3_08D3;
const DIGEST_SEED_HIGH: u64 = 0x1319_8A2E_0370_7344;

/// 64 位的末轮搅拌（splitmix64 的那一步）。
fn avalanche_word(word: u64) -> u64 {
    let mut mixed = word ^ (word >> 30);
    mixed = mixed.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed ^= mixed >> 27;
    mixed = mixed.wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^ (mixed >> 31)
}

/// 逐个吃 64 位字、最后出 128 位摘要。`domain_tag` 把不同种类的输入分开（同样的字按不同种类喂出不同摘要）。
struct DigestBuilder {
    low: u64,
    high: u64,
    words_fed: u64,
}

impl DigestBuilder {
    fn new(domain_tag: u64) -> Self {
        Self {
            low: DIGEST_SEED_LOW ^ domain_tag,
            high: DIGEST_SEED_HIGH ^ domain_tag.rotate_left(32),
            words_fed: 0,
        }
    }

    fn feed_word(&mut self, word: u64) {
        self.low = (self.low ^ word)
            .wrapping_mul(DIGEST_MULTIPLIER_LOW)
            .rotate_left(29);
        self.high = ((self.high ^ word.rotate_left(17)).wrapping_mul(DIGEST_MULTIPLIER_HIGH))
            .rotate_left(37)
            ^ self.low;
        self.words_fed += 1;
    }

    fn feed_digest(&mut self, digest: Digest128) {
        self.feed_word(u64::try_from(digest.0 & u128::from(u64::MAX)).expect("低 64 位"));
        self.feed_word(u64::try_from(digest.0 >> 64).expect("高 64 位"));
    }

    /// 先喂长度，再按 8 字节小端逐字喂，尾巴补零成一个字。
    fn feed_bytes(&mut self, bytes: &[u8]) {
        self.feed_word(u64::try_from(bytes.len()).expect("长度装得进 u64"));
        let (words, tail) = bytes.as_chunks::<8>();
        for word in words {
            self.feed_word(u64::from_le_bytes(*word));
        }
        if !tail.is_empty() {
            let mut padded = [0u8; 8];
            padded[..tail.len()].copy_from_slice(tail);
            self.feed_word(u64::from_le_bytes(padded));
        }
    }

    fn finish(self) -> Digest128 {
        let low = avalanche_word(self.low ^ self.words_fed ^ self.high.rotate_left(13));
        let high = avalanche_word(self.high ^ low.rotate_left(7) ^ self.words_fed.rotate_left(40));
        Digest128((u128::from(high) << 64) | u128::from(low))
    }
}

/// 摘要的种类标签：内容、读集调用、键、结局指纹各自分开。
const DIGEST_TAG_CONTENT: u64 = 0x636F_6E74_656E_7431;
const DIGEST_TAG_CALL: u64 = 0x6361_6C6C_3132_3831;
const DIGEST_TAG_KEY_SEQUENCE: u64 = 0x6B65_7973_6571_3131;
const DIGEST_TAG_KEY_SET: u64 = 0x6B65_7973_6574_3131;
const DIGEST_TAG_OUTCOME: u64 = 0x6F75_7463_6F6D_6531;
const DIGEST_TAG_PAIR: u64 = 0x7061_6972_3132_3831;
const DIGEST_TAG_POSITION: u64 = 0x706F_7369_7469_6F6E;

fn digest_of_bytes(bytes: &[u8]) -> Digest128 {
    let mut builder = DigestBuilder::new(DIGEST_TAG_CONTENT);
    builder.feed_bytes(bytes);
    builder.finish()
}

/// 结局的指纹：`Debug` 文本的摘要（`RecoveryReport` 与判定表都派生了 `Debug`，逐字段都在文本里；
/// 文本相同 ⇔ 值相同，除去 128 位摘要撞车，概率上界见登记 7.3）。
fn fingerprint_of_debug_text(value: &dyn std::fmt::Debug) -> Digest128 {
    let mut builder = DigestBuilder::new(DIGEST_TAG_OUTCOME);
    builder.feed_bytes(format!("{value:?}").as_bytes());
    builder.finish()
}

// ───────────────────────────── 两条流的搭建（S2） ─────────────────────────────

/// 哪一条流（登记第一节「读法写死」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum StreamName {
    /// 层 0 第一条流：mkfs → 取号 → 暖机两次 → 第一个事务。
    First,
    /// 层 0 第二条流：固定脚本到 E 再正常卸载，取段 9–22。
    Second,
}

impl StreamName {
    fn label(self) -> &'static str {
        match self {
            Self::First => "first",
            Self::Second => "second",
        }
    }
}

/// 一条流搭好之后的样子：基镜像（mkfs 之后）、录制流的写表与段、被判的根、版本表。
struct PreparedStream {
    name: StreamName,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    root_write_count: usize,
}

/// 第一条流：照 `tests/first_transaction_step_seven_layer0.rs:450-481` 的 `prepare`（S2 的段长与写数断言照它）。
fn prepare_first_stream() -> PreparedStream {
    let pool = common::build_pool("e161-first");
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) = singlefs_harness::crash::writes_and_segments(
        &operations[pool.mkfs_operation_count..],
        &common::geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("写流里有根槽那一条");
    let (instance, checkpoint_txg) = root_identity_of_root_write(&writes[judged_root_index]);
    let root_write_count = writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .count();
    PreparedStream {
        name: StreamName::First,
        base,
        writes,
        segments,
        judged_root_index,
        versions: vec![PublishedVersion {
            instance,
            checkpoint_txg,
            content: common::file_content(),
        }],
        root_write_count,
    }
}

/// 根槽写写下的根身份：实例代号在偏移 24、checkpoint_txg 在偏移 28（`crash.rs` 的 `root_identity_written_by` 同一读法）。
fn root_identity_of_root_write(write: &RetainedWrite) -> (InstanceGeneration, CheckpointTxg) {
    let bytes = write.bytes().expect("根槽写带着字节");
    (
        InstanceGeneration(u32::from_le_bytes(
            bytes[24..28].try_into().expect("4 字节"),
        )),
        CheckpointTxg(u64::from_le_bytes(
            bytes[28..36].try_into().expect("8 字节"),
        )),
    )
}

/// 第二条流的内容与脚本照抄 `second_transaction_step_zero_layer0.rs:45-51`、`116-149`、`153-201`。
const SECOND_FILE_BYTES: usize = 4100;

fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

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

fn overwrite(
    pool: &mut common::BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> singlefs_core::transaction::TransactionOutput {
    let parameters = common::parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer =
        singlefs_core::transaction::PoolWriter::new(&parameters, devices.as_mut_slice());
    singlefs_core::transaction::publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &pool.output,
        singlefs_core::transaction::FirstFile {
            content,
            write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写")
}

fn unmount_recorded(pool: &mut common::BuiltPool) -> singlefs_core::mount::UnmountRaisedTheFloor {
    let mut current = singlefs_core::transaction::PoolVersion::WithFile(pool.output.clone());
    let stream = pool.stream.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let allocator = &mut pool.allocator;
    let unmounted = stream
        .record_entry(singlefs_harness::RecordedPublishEntry::Unmount, || {
            singlefs_core::mount::unmount(
                &common::parameters(),
                devices,
                allocator,
                &mut current,
                singlefs_core::mount::ShadowLedger::On,
            )
        })
        .expect("正常卸载");
    pool.output = current
        .into_file_version()
        .expect("带文件的一版卸载之后仍带文件");
    match unmounted {
        singlefs_core::mount::Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised,
        singlefs_core::mount::Unmounted::NothingWrittenOnAVersionWithoutFile {
            current: reported_version,
        } => panic!("带文件的一版上卸载报成「一个字节都不写」：{reported_version:?}"),
    }
}

fn writes_and_segments_after_mkfs(
    pool: &common::BuiltPool,
) -> (Vec<RetainedWrite>, Vec<Vec<usize>>) {
    let operations = pool.retained_operations();
    let entry_spans: Vec<singlefs_harness::RecordedEntrySpan> = pool
        .stream
        .entry_spans()
        .into_iter()
        .map(|span| singlefs_harness::RecordedEntrySpan {
            entry: span.entry,
            operations: span.operations.start - pool.mkfs_operation_count
                ..span.operations.end - pool.mkfs_operation_count,
        })
        .collect();
    let (writes, segments, _stream_indexes) =
        singlefs_harness::crash::writes_and_segments_with_stream_indexes_and_entries(
            &operations[pool.mkfs_operation_count..],
            &entry_spans,
            &common::geometry(),
        );
    (writes, segments)
}

fn version(instance: u32, txg: u64, content: Vec<u8>) -> PublishedVersion {
    PublishedVersion {
        instance: InstanceGeneration(instance),
        checkpoint_txg: CheckpointTxg(txg),
        content,
    }
}

/// 第二条流：`second_transaction_step_zero_layer0.rs:203-474` 的 `prepare`，`Script::ReuseAfterRaisingFloorThenNormalUnmount` 那一支。
/// 脚本里的断言（E 的数据单元落在 50176、卸载两次空发布是 txg 18、19）原样留着，S2 另核段长表、写数与根数。
fn prepare_second_stream() -> PreparedStream {
    let mut pool = common::build_pool("e161-second");
    let second = overwrite(&mut pool, &second_content(), InstanceGeneration(1));
    let mut versions = vec![
        version(1, 3, common::file_content()),
        version(1, 4, second_content()),
    ];
    pool.output = second;
    let mut devices = pool.reopen_recorded();
    let mounted = singlefs_core::mount::mount_writable(&common::parameters(), &mut devices)
        .expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    for txg in 5..=7 {
        versions.push(version(2, txg, second_content()));
    }
    let third = overwrite(&mut pool, &third_content(), InstanceGeneration(2));
    versions.push(version(2, 8, third_content()));
    pool.output = third;
    {
        let rollback_parameters = common::parameters();
        let open_devices = pool.devices.as_mut().expect("镜像还开着");
        singlefs_core::mount::roll_back_by_a_forward_publish(
            &rollback_parameters,
            open_devices,
            &mut pool.allocator,
            &mut pool.output,
            singlefs_core::mount::RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
            },
        )
        .expect("挂着的时候回退到 A");
    }
    versions.push(version(2, 9, common::file_content()));
    let mut latest = Vec::new();
    for (txg, seed) in [(10u64, 13usize), (11, 17), (12, 19), (13, 23), (14, 29)] {
        latest = later_content(seed);
        pool.output = overwrite(&mut pool, &latest, InstanceGeneration(2));
        versions.push(version(2, txg, latest.clone()));
    }
    let mut current = pool.output.clone();
    let raise_devices = pool.devices.as_mut().expect("镜像还开着");
    let raised = singlefs_core::mount::raise_rollback_floor(
        &common::parameters(),
        raise_devices,
        &mut pool.allocator,
        &mut current,
        CheckpointTxg(11),
        singlefs_core::mount::ShadowLedger::On,
    )
    .expect("抬 F");
    assert_eq!(raised.publishes.len(), 2, "txg 15 落盘 0、txg 16 落盘 1");
    pool.output = current;
    for txg in 15..=16 {
        versions.push(version(2, txg, latest.clone()));
    }
    let reuse = overwrite(&mut pool, &later_content(31), InstanceGeneration(2));
    assert_eq!(
        reuse.data_pointers[0].locations[0].slot.0, 50176,
        "S2：E 的数据单元落回 50176"
    );
    versions.push(version(2, 17, later_content(31)));
    pool.output = reuse;
    let unmounted = unmount_recorded(&mut pool);
    assert_eq!(
        unmounted
            .publishes
            .iter()
            .map(|publish| publish.root.checkpoint_txg)
            .collect::<Vec<_>>(),
        vec![CheckpointTxg(18), CheckpointTxg(19)],
        "S2：卸载推两次空发布 txg 18、19"
    );
    for txg in 18..=19 {
        versions.push(version(2, txg, later_content(31)));
    }
    let base = pool.memory_pool_after_mkfs();
    let (writes, segments) = writes_and_segments_after_mkfs(&pool);
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    PreparedStream {
        name: StreamName::Second,
        base,
        judged_root_index: *root_indexes.last().expect("至少一条根槽写"),
        root_write_count: root_indexes.len(),
        writes,
        segments,
        versions,
    }
}

/// S2 的锚点：第一条流段长 `[2,2,1,2,2,1,26,2,1,2]`、41 次写（`first_transaction_step_seven_layer0.rs:457-463`）；
/// 第二条流 64 段的段长表、477 次写、19 条根（`second_transaction_step_zero_layer0.rs:416-465`）。
const FIRST_STREAM_SEGMENT_LENGTHS: [usize; 10] = [2, 2, 1, 2, 2, 1, 26, 2, 1, 2];
const FIRST_STREAM_WRITES: usize = 41;
/// 第一条流的根槽写：暖机两次各一条、第一个事务一条（段表里三个 1 写段，收严：登记 S2 没列这一格）。
const FIRST_STREAM_ROOTS: usize = 3;
const SECOND_STREAM_SEGMENT_LENGTHS: [usize; 64] = [
    2, 2, 1, 2, 2, 1, 26, 2, 1, 26, 2, 1, 2, 2, 18, 2, 1, 18, 2, 1, 18, 2, 1, 26, 2, 1, 22, 2, 1,
    30, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 2, 2, 16, 2, 1, 18, 2, 1, 30, 2, 1, 2, 2, 16,
    2, 1, 18, 2, 1, 2,
];
const SECOND_STREAM_WRITES: usize = 477;
const SECOND_STREAM_ROOTS: usize = 19;
/// 第二条流取段 9–22（第一节「读法写死」）；段 0–8 与第一条流段 0–8 逐字节相同（S2）。
const SECOND_STREAM_FIRST_SAMPLED_SEGMENT: usize = 9;
const SECOND_STREAM_LAST_SAMPLED_SEGMENT: usize = 22;
const SHARED_PREFIX_SEGMENTS: usize = 9;

/// S2 的一行：每条判据各自一个布尔。
struct StreamCheck {
    segment_lengths_match: bool,
    writes_match: bool,
    roots_match: bool,
}

fn check_stream_shape(stream: &PreparedStream) -> StreamCheck {
    let lengths: Vec<usize> = stream.segments.iter().map(Vec::len).collect();
    match stream.name {
        StreamName::First => StreamCheck {
            segment_lengths_match: lengths == FIRST_STREAM_SEGMENT_LENGTHS,
            writes_match: stream.writes.len() == FIRST_STREAM_WRITES,
            roots_match: stream.root_write_count == FIRST_STREAM_ROOTS,
        },
        StreamName::Second => StreamCheck {
            segment_lengths_match: lengths == SECOND_STREAM_SEGMENT_LENGTHS,
            writes_match: stream.writes.len() == SECOND_STREAM_WRITES,
            roots_match: stream.root_write_count == SECOND_STREAM_ROOTS,
        },
    }
}

/// S2：两条流的基镜像逐字节相同；第二条流段 0–8 的写与第一条流段 0–8 逐字节相同。
fn shared_prefix_is_identical(first: &PreparedStream, second: &PreparedStream) -> (bool, bool) {
    let bases_identical = first.base == second.base;
    let prefix_of = |stream: &PreparedStream| -> Vec<RetainedWrite> {
        stream.segments[..SHARED_PREFIX_SEGMENTS]
            .iter()
            .flatten()
            .map(|write_index| stream.writes[*write_index].clone())
            .collect()
    };
    (bases_identical, prefix_of(first) == prefix_of(second))
}

// ───────────────────────────── preflight 与入口 ─────────────────────────────

/// 准入与运行条件（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）：起 `preflight.py check`，
/// 退出码不是 0 就原样退出；交回 `forced` 时的摘要（强制跑的要写进产物）。
fn preflight() -> Option<String> {
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source = format!("{manifest_directory}/src/bin/e161_crash_state_dedup_and_time_split.rs");
    let script = format!("{manifest_directory}/../../.claude/singlefs-ai-sop/scripts/preflight.py");
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut command = std::process::Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
    let first_line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    first_line
        .strip_prefix("forced")
        .map(|summary| summary.trim().replace(' ', "_"))
}

fn main() {
    let forced_summary = preflight();
    let arguments: Vec<String> = std::env::args()
        .skip(1)
        .filter(|argument| argument != "--force")
        .collect();
    let mode = arguments.first().map(String::as_str);
    let mut output = ResultLines::new();
    if let Some(summary) = forced_summary {
        output.line(format!("name=preflight forced={summary}"));
    }
    let exit_code = match mode {
        Some("segment-one") => run_segment_one(&mut output),
        Some("probe") => run_probe(&mut output, arguments.get(1).map(String::as_str)),
        Some("stop-clauses") => run_stop_clauses(&mut output),
        Some("feasibility") => run_feasibility(&mut output),
        Some("probe-first-stream-stop-clauses") => run_first_stream_stop_clause_probe(&mut output),
        Some(_) | None => {
            eprintln!("  ✗ 用法：e161_crash_state_dedup_and_time_split stop-clauses | segment-one | probe <状态数> | probe-first-stream-stop-clauses");
            eprintln!(
                "  → 怎么办：第一段照登记 5.5 跑 segment-one；probe 只量每状态耗时，不出产物"
            );
            2
        }
    };
    output.finish();
    std::process::exit(exit_code);
}

/// 产物行：每行 `E7RESULT ` 起头，收尾 `name=done emitted=N`（N 连 config 行与 done 行自己都算）。
struct ResultLines {
    emitted: u64,
}

impl ResultLines {
    fn new() -> Self {
        Self { emitted: 0 }
    }

    fn line(&mut self, text: String) {
        self.emitted += 1;
        println!("E7RESULT {text}");
    }

    fn finish(&mut self) {
        self.emitted += 1;
        println!("E7RESULT name=done emitted={}", self.emitted);
    }
}

fn run_probe(output: &mut ResultLines, _states: Option<&str>) -> i32 {
    let first = prepare_first_stream();
    let second = prepare_second_stream();
    let first_check = check_stream_shape(&first);
    let second_check = check_stream_shape(&second);
    let (bases_identical, prefix_identical) = shared_prefix_is_identical(&first, &second);
    output.line(format!(
        "name=probe first_segments_ok={} first_writes_ok={} first_roots_ok={} second_segments_ok={} second_writes_ok={} second_roots_ok={} bases_identical={bases_identical} prefix_identical={prefix_identical}",
        first_check.segment_lengths_match,
        first_check.writes_match,
        first_check.roots_match,
        second_check.segment_lengths_match,
        second_check.writes_match,
        second_check.roots_match,
    ));
    let requested: usize = _states.and_then(|text| text.parse().ok()).unwrap_or(1000);
    for stream in [&first, &second] {
        let table = TornWriteTable::of(stream);
        let plan = LocalPlan::new(
            stream,
            &table,
            domain_expansions(stream, LocalExpansion::EveryCombination),
        );
        let context = EvaluationContext::new(stream, &table);
        let small = small_domain_ordinals(stream, &plan);
        let stride = stride_domain_ordinals(stream, &plan);
        let mut all: Vec<u64> = small.iter().chain(&stride).copied().collect();
        all.sort_unstable();
        let control = control_ordinals(stream, &plan, &all);
        for (label, ordinals) in [("small", &small), ("stride", &stride)] {
            let taken: Vec<u64> = ordinals.iter().copied().take(requested).collect();
            let started = Instant::now();
            let totals = run_cell(&CellRun {
                label: format!("probe-{}-{label}", stream.name.label()),
                context: &context,
                plan: &plan,
                ordinals: &taken,
                control_ordinals: &control,
                busy_zone: None,
                threads: threads_from_environment(),
                depth: EvaluationDepth::FourSteps,
            });
            let control_calls: usize = totals
                .control
                .iter()
                .map(|data| {
                    data.consult_calls.len() + data.ignore_calls.len() + data.full_calls.len()
                })
                .sum();
            output.line(format!(
                "name=probe_cell stream={} domain={label} domain_states={} probed={} wall_elapsed_ns={} consult_differs={} ignore_differs={} verdicts_differ={} unit_events={} unit_positions={} control_states={} control_calls={control_calls} replays={} w={}",
                stream.name.label(),
                ordinals.len(),
                totals.states(),
                started.elapsed().as_nanos(),
                totals.consult_report_differs,
                totals.ignore_report_differs,
                totals.verdicts_differ,
                totals.whole_unit_events,
                totals.whole_unit_positions,
                totals.control.len(),
                table.replay_count,
                table.mask_width(),
            ));
        }
    }
    0
}

fn run_segment_one(output: &mut ResultLines) -> i32 {
    let threads = threads_from_environment();
    output.line(format!(
        "name=config experiment=E161 mode=segment_one threads={threads} registration=research/prompts/e161-preregistration.md stop_clauses_product=research/results/e161-crash-state-dedup-and-time-split-stop-clauses-2026-09-27.out"
    ));
    let constants_hold = local_constants_against_the_crates()
        .iter()
        .all(|(_name, local, crates)| local == crates);
    let first = prepare_first_stream();
    let second = prepare_second_stream();
    let first_shape = check_stream_shape(&first);
    let second_shape = check_stream_shape(&second);
    let shapes_hold = first_shape.segment_lengths_match
        && first_shape.writes_match
        && second_shape.segment_lengths_match
        && second_shape.writes_match;
    output.line(format!(
        "name=stop_clause_recheck s1_constants_hold={} s2_shapes_hold={}",
        bool_text(constants_hold),
        bool_text(shapes_hold)
    ));
    if !(constants_hold && shapes_hold) {
        return 3;
    }
    let (first_outcome, first_discriminating) = process_stream(output, &first, threads);
    let (second_outcome, second_discriminating) = process_stream(output, &second, threads);
    // 8.2 线程数那一格：第一条流的 D1-stride 用线程数 1 再跑一遍，全部计数与结果行（去掉计时字段）逐字比（V6）。
    let first_table = TornWriteTable::of(&first);
    let first_context = EvaluationContext::new(&first, &first_table);
    let first_full = LocalPlan::new(
        &first,
        &first_table,
        domain_expansions(&first, LocalExpansion::EveryCombination),
    );
    let first_stride = stride_domain_ordinals(&first, &first_full);
    let strided: BTreeSet<usize> = first_stride
        .iter()
        .map(|ordinal| first_full.segment_of(*ordinal))
        .collect();
    let mut first_small_and_stride: Vec<u64> = small_domain_ordinals(&first, &first_full)
        .into_iter()
        .chain(first_stride.iter().copied())
        .collect();
    first_small_and_stride.sort_unstable();
    let first_control = control_ordinals(&first, &first_full, &first_small_and_stride);
    let mut single_thread = run_cell(&CellRun {
        label: "first_stride_threads_one".to_string(),
        context: &first_context,
        plan: &first_full,
        ordinals: &first_stride,
        control_ordinals: &first_control,
        busy_zone: None,
        threads: 1,
        depth: EvaluationDepth::FourSteps,
    });
    let single_thread_stores = std::mem::take(&mut single_thread.arm_stores);
    let single_thread_summary = report_cell(
        output,
        "first_stride",
        &single_thread,
        single_thread_stores,
        &TrajectoryIntervals::of(&first_full, &first_stride, &strided),
        None,
    );
    let lines_identical =
        single_thread_summary.lines_without_timing == first_outcome.stride.lines_without_timing;
    output.line(format!(
        "name=geometry_sensitivity knob=threads stream=first cell=first_stride threads_default={threads} threads_other=1 result_lines_identical_without_timing={} {}",
        bool_text(lines_identical),
        share_sensitivity_fields(
            "unit_share",
            unweighted_unit_share(&first_outcome.stride),
            unweighted_unit_share(&single_thread_summary)
        ),
    ));
    // 8.2 掩码宽那一格：第一条流的 D1 用第二条流整条流的 W 重判 Q5b、Q5c。
    for arm in [Arm::WalkUnits, Arm::AllUnits, Arm::FullReadSet] {
        let mut relabelled = first_outcome.union.clone();
        relabelled.label = "first_d1_with_second_stream_w".to_string();
        let _verdicts = report_device_memory(
            output,
            &relabelled,
            second_outcome.write_table_length,
            second_outcome.write_table_length,
            arm,
        );
    }
    report_geometry_sensitivity(output, &first_outcome);
    report_geometry_sensitivity(output, &second_outcome);
    let timing_control_holds = report_timing_control(output, &second, threads);
    report_verdicts(
        output,
        &[&first_outcome, &second_outcome],
        &[first_discriminating, second_discriminating],
        timing_control_holds,
        lines_identical,
    )
}

// ───────────────────────────── 自己的枚举计划与撕裂镜像表（S3、A1） ─────────────────────────────

/// 一段怎么展开（本文件自己的一份，与 crates 的 `Layer0SegmentExpansion` 在 S3 里逐段对拍状态数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LocalExpansion {
    /// 不展开：只以整段持久进入后面的状态。
    Skipped,
    /// 全量：段内每次写各取它的几态、任意组合，去掉整段全持久。
    EveryCombination,
    /// 甲二：原地写各取几态、任意组合 × 单元写全不落或全落，去掉整段全落。
    InPlaceCombinationsWithUnitWritesNoneOrAll,
}

impl LocalExpansion {
    fn crates_expansion(self) -> Layer0SegmentExpansion {
        match self {
            Self::Skipped => Layer0SegmentExpansion::NotExpanded,
            Self::EveryCombination => Layer0SegmentExpansion::EveryProperSubset,
            Self::InPlaceCombinationsWithUnitWritesNoneOrAll => {
                Layer0SegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll
            }
        }
    }
}

/// 一次写在一个崩溃状态里怎么落。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Landing {
    NotPersisted,
    Torn,
    Persisted,
}

/// 原地覆写的三条判据（D13（验证路线） 已定项 4「原地覆写（不是单元写、长于一个扇区、罩住的范围里原来有东西的写）」），
/// 本文件自己判：基镜像那一段有没有非零字节按 [`MemoryPool`] 读出来看，不碰 crates 的私有字段。
fn is_in_place_overwrite(base: &MemoryPool, writes: &[RetainedWrite], write_index: usize) -> bool {
    let write = &writes[write_index];
    let is_unit_write = match write.kind {
        StepKind::UnitWrite => true,
        StepKind::ZeroFill
        | StepKind::JournalRecord
        | StepKind::RootRecordFua
        | StepKind::SystemConfigurationSlot
        | StepKind::Barrier => false,
    };
    if is_unit_write || write.length_in_bytes() <= LOCAL_SECTOR_BYTES {
        return false;
    }
    let write_end = write.offset.0 + write.length_in_bytes();
    let base_bytes = base.devices.get(&write.device).map(|device| {
        device.read(
            write.offset,
            usize::try_from(write.length_in_bytes()).expect("写长装得进 usize"),
        )
    });
    let base_holds_nonzero = base_bytes.is_some_and(|bytes| bytes.iter().any(|byte| *byte != 0));
    let earlier_overlapping_write_with_bytes = writes[..write_index].iter().any(|earlier| {
        let has_bytes = match earlier.contents {
            WrittenContents::Bytes(_) => true,
            WrittenContents::Zeros { .. } => false,
        };
        has_bytes
            && earlier.device == write.device
            && earlier.offset.0 < write_end
            && write.offset.0 < earlier.offset.0 + earlier.length_in_bytes()
    });
    base_holds_nonzero || earlier_overlapping_write_with_bytes
}

/// 撕裂镜像的字节：新旧不同的那一截前一半新、后一半旧（D13（验证路线） 已定项 4「按字节撕」），本文件自己的一份。
fn torn_bytes(old: &[u8], new: &WrittenContents) -> Vec<u8> {
    let length = usize::try_from(new.length_in_bytes()).expect("写长装得进 usize");
    let mut new_bytes = vec![0u8; length];
    new.copy_range_into(0, &mut new_bytes);
    let differing: Vec<usize> = (0..length)
        .filter(|index| old[*index] != new_bytes[*index])
        .collect();
    let (Some(first_differing), Some(last_differing)) = (differing.first(), differing.last())
    else {
        return new_bytes;
    };
    let first_old_byte = first_differing + (last_differing - first_differing).div_ceil(2);
    let mut torn = new_bytes;
    torn[first_old_byte..].copy_from_slice(&old[first_old_byte..]);
    torn
}

/// 枚举用的写表：录制流的写在前，每个原地覆写接一条撕裂镜像、再接同段里之后与它重叠的写的重放。
struct TornWriteTable {
    writes: Vec<RetainedWrite>,
    is_in_place_overwrite: Vec<bool>,
    /// 原地覆写（录制流下标）→（撕裂镜像的下标，[(重放的下标, 被重放的录制流下标)]）。
    torn_image_by_write: BTreeMap<usize, (usize, Vec<(usize, usize)>)>,
    replay_count: usize,
}

impl TornWriteTable {
    fn of(stream: &PreparedStream) -> Self {
        let recorded = &stream.writes;
        let is_in_place_overwrite: Vec<bool> = (0..recorded.len())
            .map(|write_index| is_in_place_overwrite(&stream.base, recorded, write_index))
            .collect();
        let mut segment_of_write = BTreeMap::new();
        for (segment_index, segment) in stream.segments.iter().enumerate() {
            for write_index in segment {
                segment_of_write.insert(*write_index, segment_index);
            }
        }
        let mut writes = recorded.clone();
        let mut torn_image_by_write = BTreeMap::new();
        let mut replay_count = 0;
        let mut image_before = stream.base.clone();
        let mut next_to_apply = 0usize;
        for write_index in (0..recorded.len()).filter(|index| is_in_place_overwrite[*index]) {
            image_before.apply_writes(&recorded[next_to_apply..write_index]);
            next_to_apply = write_index;
            let write = &recorded[write_index];
            let length = usize::try_from(write.length_in_bytes()).expect("写长");
            let old = image_before
                .devices
                .get(&write.device)
                .expect("写落在池里的盘上")
                .read(write.offset, length);
            let torn_index = writes.len();
            writes.push(RetainedWrite {
                device: write.device,
                kind: write.kind,
                is_force_unit_access: write.is_force_unit_access,
                offset: write.offset,
                contents: WrittenContents::Bytes(torn_bytes(&old, &write.contents)),
            });
            let write_end = write.offset.0 + write.length_in_bytes();
            let later_overlapping: Vec<usize> = segment_of_write
                .get(&write_index)
                .map(|segment_index| {
                    stream.segments[*segment_index]
                        .iter()
                        .copied()
                        .filter(|later_index| {
                            let later = &recorded[*later_index];
                            *later_index > write_index
                                && later.device == write.device
                                && later.offset.0 < write_end
                                && write.offset.0 < later.offset.0 + later.length_in_bytes()
                        })
                        .collect()
                })
                .unwrap_or_default();
            let replays: Vec<(usize, usize)> = later_overlapping
                .into_iter()
                .map(|replayed| {
                    writes.push(recorded[replayed].clone());
                    replay_count += 1;
                    (writes.len() - 1, replayed)
                })
                .collect();
            torn_image_by_write.insert(write_index, (torn_index, replays));
        }
        Self {
            writes,
            is_in_place_overwrite,
            torn_image_by_write,
            replay_count,
        }
    }

    /// 掩码宽 W：带撕裂镜像的写表长度（A5：录制流写数 + 取三态的写数 + 重放数）。
    fn mask_width(&self) -> usize {
        self.writes.len()
    }

    fn choices_of(&self, write_index: usize) -> u64 {
        if self.is_in_place_overwrite[write_index] {
            3
        } else {
            2
        }
    }

    fn landing_of_digit(&self, write_index: usize, digit: u64) -> Landing {
        match (self.is_in_place_overwrite[write_index], digit) {
            (false, 0) | (true, 0) => Landing::NotPersisted,
            (false, 1) | (true, 2) => Landing::Persisted,
            (true, 1) => Landing::Torn,
            (_, out_of_range) => unreachable!("数字 {out_of_range} 超出这次写的态数"),
        }
    }

    fn combinations_of(&self, write_indexes: &[usize]) -> u64 {
        write_indexes
            .iter()
            .map(|write_index| self.choices_of(*write_index))
            .product()
    }
}

/// 一段里的原地写与单元写（段内次序）。
fn in_place_and_unit_writes(
    writes: &[RetainedWrite],
    segment: &[usize],
) -> (Vec<usize>, Vec<usize>) {
    segment
        .iter()
        .copied()
        .partition(|write_index| match writes[*write_index].kind {
            StepKind::UnitWrite => false,
            StepKind::ZeroFill
            | StepKind::SystemConfigurationSlot
            | StepKind::JournalRecord
            | StepKind::RootRecordFua
            | StepKind::Barrier => true,
        })
}

/// 自己的枚举计划：序号 → 段 → 段内混合进制（段里第一次写是最低位），最后可选全部持久那一个。
struct LocalPlan<'plan> {
    stream: &'plan PreparedStream,
    table: &'plan TornWriteTable,
    expansion_by_segment: Vec<LocalExpansion>,
    first_ordinal_by_segment: Vec<u64>,
    count_by_segment: Vec<u64>,
    /// 各段展开出来的状态数之和（不含全部持久那一个）。
    expanded_state_count: u64,
}

impl<'plan> LocalPlan<'plan> {
    fn new(
        stream: &'plan PreparedStream,
        table: &'plan TornWriteTable,
        expansion_by_segment: Vec<LocalExpansion>,
    ) -> Self {
        let count_by_segment: Vec<u64> = stream
            .segments
            .iter()
            .zip(&expansion_by_segment)
            .map(|(segment, expansion)| {
                Self::segment_state_count(stream, table, segment, *expansion)
            })
            .collect();
        let mut first_ordinal_by_segment = Vec::new();
        let mut next = 0u64;
        for count in &count_by_segment {
            first_ordinal_by_segment.push(next);
            next += count;
        }
        Self {
            stream,
            table,
            expansion_by_segment,
            first_ordinal_by_segment,
            count_by_segment,
            expanded_state_count: next,
        }
    }

    fn segment_state_count(
        stream: &PreparedStream,
        table: &TornWriteTable,
        segment: &[usize],
        expansion: LocalExpansion,
    ) -> u64 {
        match expansion {
            LocalExpansion::Skipped => 0,
            LocalExpansion::EveryCombination => table.combinations_of(segment) - 1,
            LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll => {
                let (in_place, unit_writes) = in_place_and_unit_writes(&stream.writes, segment);
                let unit_choices = if unit_writes.is_empty() { 1 } else { 2 };
                table.combinations_of(&in_place) * unit_choices - 1
            }
        }
    }

    /// 序号落在哪一段；等于段数说明是全部持久那一个。
    fn segment_of(&self, ordinal: u64) -> usize {
        self.first_ordinal_by_segment
            .iter()
            .zip(&self.count_by_segment)
            .position(|(first, count)| ordinal >= *first && ordinal < first + count)
            .unwrap_or(self.stream.segments.len())
    }

    fn assign(&self, landings: &mut [Landing], ordinal_within: u64, write_indexes: &[usize]) {
        let mut remaining = ordinal_within;
        for write_index in write_indexes {
            let choices = self.table.choices_of(*write_index);
            landings[*write_index] = self
                .table
                .landing_of_digit(*write_index, remaining % choices);
            remaining /= choices;
        }
        assert_eq!(
            remaining, 0,
            "段内序号 {ordinal_within} 超出这几次写的组合数"
        );
    }

    /// 这个状态的持久集合，按枚举用的写表给。
    fn persisted_of(&self, ordinal: u64) -> Vec<bool> {
        let recorded_count = self.stream.writes.len();
        let mut landings = vec![Landing::NotPersisted; recorded_count];
        let segment_index = self.segment_of(ordinal);
        for segment in &self.stream.segments[..segment_index] {
            for write_index in segment {
                landings[*write_index] = Landing::Persisted;
            }
        }
        if let Some(segment) = self.stream.segments.get(segment_index) {
            let within = ordinal - self.first_ordinal_by_segment[segment_index];
            match self.expansion_by_segment[segment_index] {
                LocalExpansion::Skipped => unreachable!("不展开的段没有状态"),
                LocalExpansion::EveryCombination => self.assign(&mut landings, within, segment),
                LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll => {
                    let (in_place, unit_writes) =
                        in_place_and_unit_writes(&self.stream.writes, segment);
                    if unit_writes.is_empty() {
                        self.assign(&mut landings, within, &in_place);
                    } else {
                        self.assign(&mut landings, within / 2, &in_place);
                        if within % 2 == 1 {
                            for write_index in &unit_writes {
                                landings[*write_index] = Landing::Persisted;
                            }
                        }
                    }
                }
            }
        }
        let mut persisted = vec![false; self.table.mask_width()];
        for (write_index, landing) in landings.iter().enumerate() {
            persisted[write_index] = *landing == Landing::Persisted;
        }
        for (write_index, (torn_index, replays)) in &self.table.torn_image_by_write {
            if landings[*write_index] == Landing::Torn {
                persisted[*torn_index] = true;
                for (replay_index, replayed) in replays {
                    persisted[*replay_index] = landings[*replayed] == Landing::Persisted;
                }
            }
        }
        persisted
    }
}

/// A1 的公式：3^m · 2^(n−m) − 1。
fn formula_segment_state_count(writes_in_segment: u64, in_place_overwrites: u64) -> u64 {
    3u64.pow(u32::try_from(in_place_overwrites).expect("m"))
        * 2u64.pow(u32::try_from(writes_in_segment - in_place_overwrites).expect("n − m"))
        - 1
}

// ───────────────────────────── 读的分类与内容表（G1、G5 P1、V2） ─────────────────────────────

/// 一次读落在哪一类（G5 P1 的四类；G1 的「单元」是单元区里 16384 或 32768 长的读）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ReadCategory {
    /// 环起点之前：系统配置槽与根槽。
    FixedStructure,
    /// journal 环里。
    JournalRing,
    /// 单元区里、整单元长（16384 或 32768）。
    WholeUnit,
    /// 单元区里、别的长度（头扫描）。
    UnitHeaderScan,
}

const READ_CATEGORIES: [ReadCategory; 4] = [
    ReadCategory::FixedStructure,
    ReadCategory::JournalRing,
    ReadCategory::WholeUnit,
    ReadCategory::UnitHeaderScan,
];

impl ReadCategory {
    fn of(offset: u64, length: usize) -> Self {
        let length_in_bytes = u64::try_from(length).expect("读长");
        if offset < RING_START_BYTES {
            Self::FixedStructure
        } else if offset < UNIT_AREA_START_BYTES {
            Self::JournalRing
        } else if length_in_bytes == LOCAL_NODE_BYTES || length_in_bytes == LOCAL_DATA_UNIT_BYTES {
            Self::WholeUnit
        } else {
            Self::UnitHeaderScan
        }
    }

    fn slot(self) -> usize {
        match self {
            Self::FixedStructure => 0,
            Self::JournalRing => 1,
            Self::WholeUnit => 2,
            Self::UnitHeaderScan => 3,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::FixedStructure => "fixed_structure",
            Self::JournalRing => "journal_ring",
            Self::WholeUnit => "whole_unit",
            Self::UnitHeaderScan => "unit_header_scan",
        }
    }
}

/// 一份内容在哪些读里出现过：`any_pass` 是恢复两遍与 checker 正常那一遍（P1），`checker_normal_pass` 只算 checker 正常那一遍
/// （G1 的 Q1a、Q1e、Q1f）；各记最小序号（轨迹按「新出现」记在序号最小的那个状态所在的区间，8.1）。
struct ContentEntry {
    bytes: Vec<u8>,
    first_ordinal_any_pass: Option<u64>,
    first_ordinal_checker_normal_pass: Option<u64>,
}

/// 按摘要分桶、桶里逐字节比的内容表（5.1）；摘要相同而字节不同记一次撞车（V2）。
#[derive(Default)]
struct ContentTable {
    entries: HashMap<Digest128, ContentEntry>,
    collisions: u64,
}

fn minimum_ordinal(current: Option<u64>, candidate: Option<u64>) -> Option<u64> {
    match (current, candidate) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(left), None) => Some(left),
        (None, right) => right,
    }
}

impl ContentTable {
    fn insert(
        &mut self,
        digest: Digest128,
        bytes: &[u8],
        ordinal: u64,
        is_checker_normal_pass: bool,
    ) {
        let checker_ordinal = is_checker_normal_pass.then_some(ordinal);
        match self.entries.get_mut(&digest) {
            Some(entry) => {
                if entry.bytes != bytes {
                    self.collisions += 1;
                }
                entry.first_ordinal_any_pass =
                    minimum_ordinal(entry.first_ordinal_any_pass, Some(ordinal));
                entry.first_ordinal_checker_normal_pass =
                    minimum_ordinal(entry.first_ordinal_checker_normal_pass, checker_ordinal);
            }
            None => {
                self.entries.insert(
                    digest,
                    ContentEntry {
                        bytes: bytes.to_vec(),
                        first_ordinal_any_pass: Some(ordinal),
                        first_ordinal_checker_normal_pass: checker_ordinal,
                    },
                );
            }
        }
    }

    fn absorb(&mut self, other: ContentTable) {
        self.collisions += other.collisions;
        for (digest, entry) in other.entries {
            match self.entries.get_mut(&digest) {
                Some(existing) => {
                    if existing.bytes != entry.bytes {
                        self.collisions += 1;
                    }
                    existing.first_ordinal_any_pass = minimum_ordinal(
                        existing.first_ordinal_any_pass,
                        entry.first_ordinal_any_pass,
                    );
                    existing.first_ordinal_checker_normal_pass = minimum_ordinal(
                        existing.first_ordinal_checker_normal_pass,
                        entry.first_ordinal_checker_normal_pass,
                    );
                }
                None => {
                    self.entries.insert(digest, entry);
                }
            }
        }
    }
}

/// 四类各一张表。
#[derive(Default)]
struct ContentTables {
    by_category: [ContentTable; 4],
}

impl ContentTables {
    fn insert(
        &mut self,
        category: ReadCategory,
        digest: Digest128,
        bytes: &[u8],
        ordinal: u64,
        is_checker_normal_pass: bool,
    ) {
        self.by_category[category.slot()].insert(digest, bytes, ordinal, is_checker_normal_pass);
    }

    fn absorb(&mut self, other: ContentTables) {
        for (mine, theirs) in self.by_category.iter_mut().zip(other.by_category) {
            mine.absorb(theirs);
        }
    }

    fn collisions(&self) -> u64 {
        self.by_category.iter().map(|table| table.collisions).sum()
    }
}

// ───────────────────────────── 记录用的读者（G2、G4 的读集） ─────────────────────────────

/// 读集里的一次调用：位置（调用种类 + 参数，PC2c / PC4c 按它去掉）与整次调用（位置 + 答案）的摘要。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LoggedCall {
    position: Digest128,
    digest: Digest128,
    description: PositionDescription,
}

/// 位置的原样（PC2c、PC4c 报前十个判别位置时给人看）：调用种类与至多三个参数（盘、偏移、长度）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PositionDescription {
    kind: u8,
    arguments: [u64; 3],
}

impl PositionDescription {
    fn of(kind: CallKind, arguments: &[u64]) -> Self {
        let mut padded = [0u64; 3];
        for (slot, argument) in padded.iter_mut().zip(arguments) {
            *slot = *argument;
        }
        Self {
            kind: kind as u8,
            arguments: padded,
        }
    }

    fn text(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.kind, self.arguments[0], self.arguments[1], self.arguments[2]
        )
    }
}

/// 一次读的原样：给单元级检查重放、G1 计数、K4 集合用。
struct LoggedRead {
    device: u32,
    offset: u64,
    length: usize,
    content: Option<(Digest128, Vec<u8>)>,
}

/// 调用种类的标签（进位置摘要）。
#[derive(Clone, Copy, Debug)]
enum CallKind {
    PoolDeviceIdentities = 1,
    PoolDeviceSize = 2,
    PoolRead = 3,
    PoolJournalHint = 4,
    ImageDevices = 11,
    ImageDeviceBytes = 12,
    ImageRead = 13,
    ImageCandidateUnitSlots = 14,
    ImageCandidateJournalSlots = 15,
}

fn position_digest(kind: CallKind, arguments: &[u64]) -> Digest128 {
    let mut builder = DigestBuilder::new(DIGEST_TAG_POSITION);
    builder.feed_word(kind as u64);
    for argument in arguments {
        builder.feed_word(*argument);
    }
    builder.finish()
}

fn call_digest(position: Digest128, answer: Option<Digest128>) -> Digest128 {
    let mut builder = DigestBuilder::new(DIGEST_TAG_CALL);
    builder.feed_digest(position);
    match answer {
        Some(digest) => {
            builder.feed_word(1);
            builder.feed_digest(digest);
        }
        None => builder.feed_word(0),
    }
    builder.finish()
}

fn digest_of_words(words: &[u64]) -> Digest128 {
    let mut builder = DigestBuilder::new(DIGEST_TAG_CONTENT);
    builder.feed_word(u64::try_from(words.len()).expect("个数"));
    for word in words {
        builder.feed_word(*word);
    }
    builder.finish()
}

/// 这一遍读者属于哪一遍：恢复两遍与 checker 正常那一遍进内容表，走树那一遍只取读集。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecordedPass {
    Recovery,
    CheckerNormal,
    CheckerWalk,
}

/// 包在崩溃镜像外面、只转发只记的读者（5.1 第 2 步）。答案一律原样交回；走树那一遍的单元候选对每块盘交回空表。
struct RecordingReader<'image, 'tables> {
    image: &'image CrashImage<'image>,
    pass: RecordedPass,
    ordinal: u64,
    tables: &'tables RefCell<ContentTables>,
    calls: RefCell<Vec<LoggedCall>>,
    reads: RefCell<Vec<LoggedRead>>,
}

impl<'image, 'tables> RecordingReader<'image, 'tables> {
    fn new(
        image: &'image CrashImage<'image>,
        pass: RecordedPass,
        ordinal: u64,
        tables: &'tables RefCell<ContentTables>,
    ) -> Self {
        Self {
            image,
            pass,
            ordinal,
            tables,
            calls: RefCell::new(Vec::new()),
            reads: RefCell::new(Vec::new()),
        }
    }

    fn log(&self, kind: CallKind, arguments: &[u64], answer: Option<Digest128>) {
        let position = position_digest(kind, arguments);
        self.calls.borrow_mut().push(LoggedCall {
            position,
            digest: call_digest(position, answer),
            description: PositionDescription::of(kind, arguments),
        });
    }

    fn log_read(
        &self,
        kind: CallKind,
        device: u32,
        offset: u64,
        length: usize,
        answer: Option<&Vec<u8>>,
    ) {
        let content = answer.map(|bytes| (digest_of_bytes(bytes), bytes.clone()));
        if let Some((digest, bytes)) = &content {
            match self.pass {
                RecordedPass::Recovery | RecordedPass::CheckerNormal => {
                    self.tables.borrow_mut().insert(
                        ReadCategory::of(offset, length),
                        *digest,
                        bytes,
                        self.ordinal,
                        self.pass == RecordedPass::CheckerNormal
                            && (ReadCategory::of(offset, length) != ReadCategory::JournalRing
                                || u64::try_from(length).expect("读长")
                                    == LOCAL_JOURNAL_RECORD_BYTES),
                    )
                }
                RecordedPass::CheckerWalk => {}
            }
        }
        self.log(
            kind,
            &[
                u64::from(device),
                offset,
                u64::try_from(length).expect("读长"),
            ],
            content.as_ref().map(|(digest, _)| *digest),
        );
        self.reads.borrow_mut().push(LoggedRead {
            device,
            offset,
            length,
            content,
        });
    }
}

impl PoolReader for RecordingReader<'_, '_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        let answer = PoolReader::device_identities(self.image);
        let words: Vec<u64> = answer
            .iter()
            .map(|identity| u64::from(identity.0))
            .collect();
        self.log(
            CallKind::PoolDeviceIdentities,
            &[],
            Some(digest_of_words(&words)),
        );
        answer
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        let answer = PoolReader::device_size_in_bytes(self.image, device);
        self.log(
            CallKind::PoolDeviceSize,
            &[u64::from(device.0)],
            answer.map(|size| digest_of_words(&[size])),
        );
        answer
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let answer = PoolReader::read(self.image, device, offset, length);
        self.log_read(
            CallKind::PoolRead,
            device.0,
            offset.0,
            length,
            answer.as_ref(),
        );
        answer
    }

    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        let answer =
            PoolReader::journal_record_offsets_hint(self.image, device, ring_start, ring_bytes);
        let words = answer
            .as_ref()
            .map(|offsets| offsets.iter().map(|offset| offset.0).collect::<Vec<u64>>());
        self.log(
            CallKind::PoolJournalHint,
            &[u64::from(device.0), ring_start.0, ring_bytes],
            words.map(|offsets| digest_of_words(&offsets)),
        );
        answer
    }
}

impl ImageReader for RecordingReader<'_, '_> {
    fn devices(&self) -> Vec<u32> {
        let answer = ImageReader::devices(self.image);
        let words: Vec<u64> = answer.iter().map(|device| u64::from(*device)).collect();
        self.log(CallKind::ImageDevices, &[], Some(digest_of_words(&words)));
        answer
    }

    fn device_bytes(&self, device: u32) -> Option<u64> {
        let answer = ImageReader::device_bytes(self.image, device);
        self.log(
            CallKind::ImageDeviceBytes,
            &[u64::from(device)],
            answer.map(|size| digest_of_words(&[size])),
        );
        answer
    }

    fn read(&self, device: u32, offset: u64, length: usize) -> Option<Vec<u8>> {
        let answer = ImageReader::read(self.image, device, offset, length);
        self.log_read(CallKind::ImageRead, device, offset, length, answer.as_ref());
        answer
    }

    fn candidate_unit_slots(&self, device: u32) -> Option<Vec<u64>> {
        let answer = match self.pass {
            RecordedPass::CheckerWalk => Some(Vec::new()),
            RecordedPass::Recovery | RecordedPass::CheckerNormal => {
                ImageReader::candidate_unit_slots(self.image, device)
            }
        };
        self.log(
            CallKind::ImageCandidateUnitSlots,
            &[u64::from(device)],
            answer.as_deref().map(digest_of_words),
        );
        answer
    }

    fn candidate_journal_slots(&self, device: u32) -> Option<Vec<u64>> {
        let answer = ImageReader::candidate_journal_slots(self.image, device);
        self.log(
            CallKind::ImageCandidateJournalSlots,
            &[u64::from(device)],
            answer.as_deref().map(digest_of_words),
        );
        answer
    }
}

// ───────────────────────────── 单元级检查重放（G3 的 Q3f、Q3g） ─────────────────────────────

/// U(字节)：指针里的整单元校验和那一道，再按字节 6 的类标签走 checker 的公开检查（第六节 G3）；结果过 `black_box`。
fn replay_unit_check(bytes: &[u8]) {
    black_box(singlefs_checker::crc32_castagnoli_table(bytes));
    match bytes.get(6).copied() {
        Some(2) => {
            let view = singlefs_checker::index_node_view(bytes);
            if let Ok(view) = &view {
                let schema = match view.key_width {
                    LOCAL_KEY_WIDTH_EXTENT => Some(singlefs_checker::KEY_SCHEMA_EXTENT),
                    LOCAL_KEY_WIDTH_INODE => Some(singlefs_checker::KEY_SCHEMA_INODE),
                    LOCAL_KEY_WIDTH_ALLOCATION => Some(singlefs_checker::KEY_SCHEMA_ALLOCATION),
                    LOCAL_KEY_WIDTH_ACCOUNTING => Some(singlefs_checker::KEY_SCHEMA_ACCOUNTING),
                    LOCAL_KEY_WIDTH_MAPPING => Some(singlefs_checker::KEY_SCHEMA_MAPPING),
                    _other_width => None,
                };
                if let Some(schema) = schema {
                    if view.level == 0 {
                        black_box(singlefs_checker::check_index_node_keys(view, schema).is_ok());
                    } else {
                        black_box(
                            singlefs_checker::check_internal_node_separators(view, schema).is_ok(),
                        );
                    }
                }
            }
            black_box(view.is_ok());
        }
        Some(3) => {
            black_box(singlefs_checker::packed_unit_view(bytes).is_ok());
        }
        Some(_) | None => {
            black_box(singlefs_checker::check_unit(bytes).is_ok());
        }
    }
}

/// G3 的六个计时区（五段 + 单元级重放），PC3 在其中一个区里加忙等。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TimingZone {
    RecoveryConsult,
    RecoveryIgnore,
    Oracle,
    PoolChecker,
    RecordChecker,
    UnitCheckReplay,
}

const TIMING_ZONES: [TimingZone; 6] = [
    TimingZone::RecoveryConsult,
    TimingZone::RecoveryIgnore,
    TimingZone::Oracle,
    TimingZone::PoolChecker,
    TimingZone::RecordChecker,
    TimingZone::UnitCheckReplay,
];

impl TimingZone {
    fn label(self) -> &'static str {
        match self {
            Self::RecoveryConsult => "recovery_consult",
            Self::RecoveryIgnore => "recovery_ignore",
            Self::Oracle => "oracle",
            Self::PoolChecker => "pool_checker",
            Self::RecordChecker => "record_checker",
            Self::UnitCheckReplay => "unit_check_replay",
        }
    }
}

fn busy_wait_if(zone: TimingZone, busy_zone: Option<TimingZone>) {
    if busy_zone == Some(zone) {
        let started = Instant::now();
        while started.elapsed() < TIMING_CONTROL_BUSY_WAIT {
            black_box(started);
        }
    }
}

fn nanoseconds_since(started: Instant) -> u128 {
    started.elapsed().as_nanos()
}

/// 一个状态的计时（纳秒）。
#[derive(Clone, Copy, Debug, Default)]
struct StateTimings {
    state_nanoseconds: u128,
    recovery_consult_nanoseconds: u128,
    recovery_ignore_nanoseconds: u128,
    oracle_nanoseconds: u128,
    pool_checker_nanoseconds: u128,
    record_checker_nanoseconds: u128,
    unit_positions_replay_nanoseconds: u128,
    unit_events_replay_nanoseconds: u128,
    walk_checker_nanoseconds: u128,
}

impl StateTimings {
    fn zone_nanoseconds(&self, zone: TimingZone) -> u128 {
        match zone {
            TimingZone::RecoveryConsult => self.recovery_consult_nanoseconds,
            TimingZone::RecoveryIgnore => self.recovery_ignore_nanoseconds,
            TimingZone::Oracle => self.oracle_nanoseconds,
            TimingZone::PoolChecker => self.pool_checker_nanoseconds,
            TimingZone::RecordChecker => self.record_checker_nanoseconds,
            TimingZone::UnitCheckReplay => self.unit_positions_replay_nanoseconds,
        }
    }

    fn five_segments_nanoseconds(&self) -> u128 {
        self.recovery_consult_nanoseconds
            + self.recovery_ignore_nanoseconds
            + self.oracle_nanoseconds
            + self.pool_checker_nanoseconds
            + self.record_checker_nanoseconds
    }
}

/// 读集按次序算键；集合（位置 + 内容）排序去重之后算键。
fn sequence_key(calls: &[LoggedCall]) -> Digest128 {
    let mut builder = DigestBuilder::new(DIGEST_TAG_KEY_SEQUENCE);
    builder.feed_word(u64::try_from(calls.len()).expect("个数"));
    for call in calls {
        builder.feed_digest(call.digest);
    }
    builder.finish()
}

/// K4-walk、K4-units 的集合：（位置摘要，位置 + 内容的摘要，位置原样），按位置排序去重。
type SetElements = BTreeSet<(Digest128, Digest128, PositionDescription)>;

fn set_key(root: Digest128, elements: &SetElements) -> Digest128 {
    set_key_of(
        root,
        elements.len(),
        elements
            .iter()
            .map(|(_position, element, _description)| *element),
    )
}

fn set_key_of(
    root: Digest128,
    element_count: usize,
    elements: impl Iterator<Item = Digest128>,
) -> Digest128 {
    let mut builder = DigestBuilder::new(DIGEST_TAG_KEY_SET);
    builder.feed_digest(root);
    builder.feed_word(u64::try_from(element_count).expect("个数"));
    for element in elements {
        builder.feed_digest(element);
    }
    builder.finish()
}

fn pair_digest(left: Digest128, right: Digest128) -> Digest128 {
    let mut builder = DigestBuilder::new(DIGEST_TAG_PAIR);
    builder.feed_digest(left);
    builder.feed_digest(right);
    builder.finish()
}

/// 单元区里的读的集合元素：（位置摘要，位置 + 内容的摘要）。读不到的也进集合（答案是「读不到」）。
fn unit_area_elements(reads: &[LoggedRead]) -> SetElements {
    reads
        .iter()
        .filter(|read| read.offset >= UNIT_AREA_START_BYTES)
        .map(|read| {
            let arguments = [
                u64::from(read.device),
                read.offset,
                u64::try_from(read.length).expect("读长"),
            ];
            let position = position_digest(CallKind::ImageRead, &arguments);
            (
                position,
                call_digest(position, read.content.as_ref().map(|(digest, _)| *digest)),
                PositionDescription::of(CallKind::ImageRead, &arguments),
            )
        })
        .collect()
}

/// G4 的「所选根」：checker 自己的择法（`chosen_system_configurations` 取第一块盘的几何、`valid_roots` 按 (txg, 实例) 取最大），
/// 记（实例、txg、根记录字节）的摘要；择不到记一个固定摘要。
fn chosen_root_digest(image: &CrashImage<'_>) -> Digest128 {
    let configurations = chosen_system_configurations(image);
    let Some((_view, geometry)) = configurations
        .into_iter()
        .find_map(|(_device, chosen)| chosen)
    else {
        return digest_of_words(&[0]);
    };
    let roots = valid_roots(image, &geometry);
    let Some((_region, _slot, root)) = roots
        .into_iter()
        .max_by_key(|(_, _, root)| (root.checkpoint_txg, root.instance))
    else {
        return digest_of_words(&[1]);
    };
    let mut builder = DigestBuilder::new(DIGEST_TAG_CONTENT);
    builder.feed_word(u64::from(root.instance));
    builder.feed_word(root.checkpoint_txg);
    builder.feed_bytes(&root.record_bytes);
    builder.finish()
}

// ───────────────────────────── 每个状态上做的四件事（5.1） ─────────────────────────────

/// G2 的两条复用臂与成对键、G4 的三条复用臂（第五节 5.2）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    RecoveryConsult,
    RecoveryIgnore,
    RecoveryPair,
    WalkUnits,
    AllUnits,
    FullReadSet,
}

const ARMS: [Arm; 6] = [
    Arm::RecoveryConsult,
    Arm::RecoveryIgnore,
    Arm::RecoveryPair,
    Arm::WalkUnits,
    Arm::AllUnits,
    Arm::FullReadSet,
];

impl Arm {
    fn slot(self) -> usize {
        match self {
            Self::RecoveryConsult => 0,
            Self::RecoveryIgnore => 1,
            Self::RecoveryPair => 2,
            Self::WalkUnits => 3,
            Self::AllUnits => 4,
            Self::FullReadSet => 5,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::RecoveryConsult => "k2_consult",
            Self::RecoveryIgnore => "k2_ignore",
            Self::RecoveryPair => "k2_pair",
            Self::WalkUnits => "k4_walk",
            Self::AllUnits => "k4_units",
            Self::FullReadSet => "k4_full",
        }
    }

    /// G5 P3 的每单元字节：读集按调用 21，集合按元素 20（第六节 G5）。
    fn bytes_per_element(self) -> u64 {
        match self {
            Self::RecoveryConsult
            | Self::RecoveryIgnore
            | Self::RecoveryPair
            | Self::FullReadSet => READ_SET_BYTES_PER_CALL,
            Self::WalkUnits | Self::AllUnits => SET_BYTES_PER_ELEMENT,
        }
    }
}

/// 一条臂上一个状态落的定长记录（5.1 第 4 步）：键、序号、结局指纹、读集或集合的大小。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct ArmRecord {
    key: Digest128,
    ordinal: u64,
    outcome: Digest128,
    elements: u32,
}

/// 对照样本上一个状态的全部读集（PC2c、PC4c 按位置去掉再算键）。
struct ControlStateData {
    ordinal: u64,
    consult_calls: Vec<LoggedCall>,
    ignore_calls: Vec<LoggedCall>,
    full_calls: Vec<LoggedCall>,
    root: Digest128,
    walk_elements: SetElements,
    unit_elements: SetElements,
    consult_outcome: Digest128,
    ignore_outcome: Digest128,
    verdict_outcome: Digest128,
}

/// 一个状态评完交回的东西（计数记进调用方给的 `Layer0Tally` 与内容表，其余在这里）。
struct StateResult {
    segment: usize,
    timings: StateTimings,
    arm_records: [ArmRecord; 6],
    consult_report_differs: bool,
    ignore_report_differs: bool,
    verdicts_differ: bool,
    whole_unit_positions: u64,
    whole_unit_events: u64,
    header_scan_positions: u64,
    header_scan_events: u64,
    ring_record_positions: u64,
    ring_record_events: u64,
    walk_reads_outside_normal_reads: bool,
    control: Option<ControlStateData>,
}

/// 评状态要的只读输入。
struct EvaluationContext<'context> {
    stream: &'context PreparedStream,
    table: &'context TornWriteTable,
    filesystem_identifier_low: u64,
}

impl EvaluationContext<'_> {
    fn new<'context>(
        stream: &'context PreparedStream,
        table: &'context TornWriteTable,
    ) -> EvaluationContext<'context> {
        EvaluationContext {
            stream,
            table,
            filesystem_identifier_low: u64::from_le_bytes(
                common::E142_FILESYSTEM_IDENTIFIER[..8]
                    .try_into()
                    .expect("8 字节"),
            ),
        }
    }
}

/// 照跑一个状态（5.1 第 1 步，真实基线）：次序与 `crash.rs` 的 `evaluate_state_recording_findings` 逐步相同，每一步各用一对
/// `Instant` 包住，外面再包一对量整个状态；计数照它记进 `tally`。交回两遍恢复的报告与判定表。
fn evaluate_baseline(
    context: &EvaluationContext<'_>,
    image: &CrashImage<'_>,
    busy_zone: Option<TimingZone>,
    timings: &mut StateTimings,
    tally: &mut Layer0Tally,
) -> (
    RecoveryReport,
    RecoveryReport,
    Vec<(&'static str, InvariantVerdict)>,
) {
    let state_started = Instant::now();
    let oracle_prefix_started = Instant::now();
    let newest = newest_persisted_root(image.writes, &image.persisted);
    let oracle_prefix_nanoseconds = nanoseconds_since(oracle_prefix_started);
    let consult_started = Instant::now();
    busy_wait_if(TimingZone::RecoveryConsult, busy_zone);
    let consulted = recover(image, JournalPolicy::Consult);
    timings.recovery_consult_nanoseconds += nanoseconds_since(consult_started);
    let ignore_started = Instant::now();
    busy_wait_if(TimingZone::RecoveryIgnore, busy_zone);
    let ignored = recover(image, JournalPolicy::Ignore);
    timings.recovery_ignore_nanoseconds += nanoseconds_since(ignore_started);
    let oracle_started = Instant::now();
    busy_wait_if(TimingZone::Oracle, busy_zone);
    let consulted_violation = singlefs_harness::crash::oracle_violation_for_versions(
        &consulted.outcome,
        consulted.effective_root,
        newest,
        &context.stream.versions,
    );
    let ignored_violation = singlefs_harness::crash::oracle_violation_for_versions(
        &ignored.outcome,
        ignored.effective_root,
        newest,
        &context.stream.versions,
    );
    timings.oracle_nanoseconds += nanoseconds_since(oracle_started) + oracle_prefix_nanoseconds;
    let checker_started = Instant::now();
    busy_wait_if(TimingZone::PoolChecker, busy_zone);
    let verdicts = check_pool_image(image);
    timings.pool_checker_nanoseconds += nanoseconds_since(checker_started);
    let records_started = Instant::now();
    busy_wait_if(TimingZone::RecordChecker, busy_zone);
    let records = check_records(image, consulted.effective_root);
    timings.record_checker_nanoseconds += nanoseconds_since(records_started);
    timings.state_nanoseconds += nanoseconds_since(state_started);
    tally.states += 1;
    if image.persisted[context.stream.judged_root_index] {
        tally.root_persisted_states += 1;
    }
    if consulted.outcome != ignored.outcome {
        tally.journal_differing_states += 1;
    }
    if consulted.journal.verification_passed + consulted.journal.verification_failed > 0 {
        tally.verification_ran_states += 1;
    }
    if consulted.journal.verification_failed > 0 {
        tally.verification_failed_states += 1;
    }
    match &consulted.outcome {
        RecoveryOutcome::NoFile { .. } => tally.no_file_states += 1,
        RecoveryOutcome::FileRead { .. } => tally.file_read_states += 1,
        RecoveryOutcome::Failed { .. } => tally.failed_states += 1,
    }
    if consulted_violation.is_some() {
        tally.violations += 1;
    }
    if ignored_violation.is_some() {
        tally.ignored_violations += 1;
    }
    for (invariant, verdict) in &verdicts {
        match verdict {
            InvariantVerdict::Holds => {
                *tally.checker_evaluated_states.entry(invariant).or_insert(0) += 1
            }
            InvariantVerdict::Violated(_) => {
                *tally.checker_evaluated_states.entry(invariant).or_insert(0) += 1;
                *tally.checker_violated_states.entry(invariant).or_insert(0) += 1;
            }
            InvariantVerdict::NotApplicable(_) => {
                *tally
                    .checker_not_applicable_states
                    .entry(invariant)
                    .or_insert(0) += 1;
            }
        }
    }
    if records.root_without_record {
        tally.record_root_without_record += 1;
    }
    if records.claimed_state_missing_unit {
        tally.record_claimed_state_missing_unit += 1;
    }
    (consulted, ignored, verdicts)
}

/// checker 正常那一遍的读按 G1 读法分出来的计数与重放用的字节。
struct CheckerReadCounts {
    whole_unit_positions: u64,
    whole_unit_events: u64,
    header_scan_positions: u64,
    header_scan_events: u64,
    ring_record_positions: u64,
    ring_record_events: u64,
}

fn count_checker_reads(reads: &[LoggedRead]) -> CheckerReadCounts {
    let mut whole_unit = BTreeSet::new();
    let mut header_scan = BTreeSet::new();
    let mut ring_records = BTreeSet::new();
    let mut counts = CheckerReadCounts {
        whole_unit_positions: 0,
        whole_unit_events: 0,
        header_scan_positions: 0,
        header_scan_events: 0,
        ring_record_positions: 0,
        ring_record_events: 0,
    };
    for read in reads.iter().filter(|read| read.content.is_some()) {
        let position = (read.device, read.offset, read.length);
        match ReadCategory::of(read.offset, read.length) {
            ReadCategory::WholeUnit => {
                counts.whole_unit_events += 1;
                whole_unit.insert(position);
            }
            ReadCategory::UnitHeaderScan => {
                counts.header_scan_events += 1;
                header_scan.insert(position);
            }
            ReadCategory::JournalRing => {
                if u64::try_from(read.length).expect("读长") == LOCAL_JOURNAL_RECORD_BYTES {
                    counts.ring_record_events += 1;
                    ring_records.insert(position);
                }
            }
            ReadCategory::FixedStructure => {}
        }
    }
    counts.whole_unit_positions = u64::try_from(whole_unit.len()).expect("个数");
    counts.header_scan_positions = u64::try_from(header_scan.len()).expect("个数");
    counts.ring_record_positions = u64::try_from(ring_records.len()).expect("个数");
    counts
}

/// Q3f：每个不同单元位置重放一次 U；Q3g：每一次单元读重放 U、每一次头扫描读算一次按位 CRC、每一次环内 4096 字节读判一次记录。
fn replay_checker_reads(
    reads: &[LoggedRead],
    filesystem_identifier_low: u64,
    busy_zone: Option<TimingZone>,
    timings: &mut StateTimings,
) {
    let positions_started = Instant::now();
    busy_wait_if(TimingZone::UnitCheckReplay, busy_zone);
    let mut replayed_positions = BTreeSet::new();
    for read in reads {
        if let (ReadCategory::WholeUnit, Some((_digest, bytes))) =
            (ReadCategory::of(read.offset, read.length), &read.content)
        {
            if replayed_positions.insert((read.device, read.offset, read.length)) {
                replay_unit_check(bytes);
            }
        }
    }
    timings.unit_positions_replay_nanoseconds += nanoseconds_since(positions_started);
    let events_started = Instant::now();
    for read in reads {
        let Some((_digest, bytes)) = &read.content else {
            continue;
        };
        match ReadCategory::of(read.offset, read.length) {
            ReadCategory::WholeUnit => replay_unit_check(bytes),
            ReadCategory::UnitHeaderScan => {
                black_box(singlefs_checker::crc32_castagnoli_bitwise(bytes));
            }
            ReadCategory::JournalRing => {
                if u64::try_from(read.length).expect("读长") == LOCAL_JOURNAL_RECORD_BYTES {
                    black_box(
                        singlefs_checker::check_journal_record(bytes, filesystem_identifier_low)
                            .is_ok(),
                    );
                }
            }
            ReadCategory::FixedStructure => {}
        }
    }
    timings.unit_events_replay_nanoseconds += nanoseconds_since(events_started);
}

/// 一个状态上的四件事（5.1）：照跑计时、记录跑、单元级检查重放、出键。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：输入、序号、段、持久集合、忙等区、要不要留对照数据、内容表、计数"
)]
fn evaluate_state(
    context: &EvaluationContext<'_>,
    ordinal: u64,
    segment: usize,
    persisted: Vec<bool>,
    busy_zone: Option<TimingZone>,
    keep_control_data: bool,
    tables: &RefCell<ContentTables>,
    tally: &mut Layer0Tally,
) -> StateResult {
    let image = CrashImage {
        base: &context.stream.base,
        writes: &context.table.writes,
        persisted,
    };
    let mut timings = StateTimings::default();
    let (consulted, ignored, verdicts) =
        evaluate_baseline(context, &image, busy_zone, &mut timings, tally);
    let consult_reader = RecordingReader::new(&image, RecordedPass::Recovery, ordinal, tables);
    let recorded_consult = recover(&consult_reader, JournalPolicy::Consult);
    let ignore_reader = RecordingReader::new(&image, RecordedPass::Recovery, ordinal, tables);
    let recorded_ignore = recover(&ignore_reader, JournalPolicy::Ignore);
    let normal_reader = RecordingReader::new(&image, RecordedPass::CheckerNormal, ordinal, tables);
    let recorded_verdicts = check_pool_image(&normal_reader);
    let walk_reader = RecordingReader::new(&image, RecordedPass::CheckerWalk, ordinal, tables);
    let walk_started = Instant::now();
    black_box(check_pool_image(&walk_reader));
    timings.walk_checker_nanoseconds += nanoseconds_since(walk_started);
    let normal_reads = normal_reader.reads.into_inner();
    replay_checker_reads(
        &normal_reads,
        context.filesystem_identifier_low,
        busy_zone,
        &mut timings,
    );
    let counts = count_checker_reads(&normal_reads);
    let root = chosen_root_digest(&image);
    let walk_elements = unit_area_elements(&walk_reader.reads.into_inner());
    let unit_elements = unit_area_elements(&normal_reads);
    let consult_calls = consult_reader.calls.into_inner();
    let ignore_calls = ignore_reader.calls.into_inner();
    let full_calls = normal_reader.calls.into_inner();
    let consult_outcome = fingerprint_of_debug_text(&consulted);
    let ignore_outcome = fingerprint_of_debug_text(&ignored);
    let verdict_outcome = fingerprint_of_debug_text(&verdicts);
    let consult_key = sequence_key(&consult_calls);
    let ignore_key = sequence_key(&ignore_calls);
    let length_of = |length: usize| u32::try_from(length).expect("读集长度装得进 u32");
    let record = |key: Digest128, outcome: Digest128, elements: usize| ArmRecord {
        key,
        ordinal,
        outcome,
        elements: length_of(elements),
    };
    let arm_records = [
        record(consult_key, consult_outcome, consult_calls.len()),
        record(ignore_key, ignore_outcome, ignore_calls.len()),
        record(
            pair_digest(consult_key, ignore_key),
            pair_digest(consult_outcome, ignore_outcome),
            consult_calls.len() + ignore_calls.len(),
        ),
        record(
            set_key(root, &walk_elements),
            verdict_outcome,
            walk_elements.len(),
        ),
        record(
            set_key(root, &unit_elements),
            verdict_outcome,
            unit_elements.len(),
        ),
        record(sequence_key(&full_calls), verdict_outcome, full_calls.len()),
    ];
    let walk_reads_outside_normal_reads = !walk_elements.is_subset(&unit_elements);
    StateResult {
        segment,
        timings,
        arm_records,
        consult_report_differs: recorded_consult != consulted,
        ignore_report_differs: recorded_ignore != ignored,
        verdicts_differ: recorded_verdicts != verdicts,
        whole_unit_positions: counts.whole_unit_positions,
        whole_unit_events: counts.whole_unit_events,
        header_scan_positions: counts.header_scan_positions,
        header_scan_events: counts.header_scan_events,
        ring_record_positions: counts.ring_record_positions,
        ring_record_events: counts.ring_record_events,
        walk_reads_outside_normal_reads,
        control: keep_control_data.then_some(ControlStateData {
            ordinal,
            consult_calls,
            ignore_calls,
            full_calls,
            root,
            walk_elements,
            unit_elements,
            consult_outcome,
            ignore_outcome,
            verdict_outcome,
        }),
    }
}

// ───────────────────────────── 定长记录的存放与外排序（5.1 第 4 步） ─────────────────────────────

/// 一条记录在溢出文件里的字节数：键 16、序号 8、结局 16、大小 4、补齐 4。
const ARM_RECORD_BYTES: usize = 48;
/// 每条臂在内存里最多放这么多条，再多就排好序写成一段溢出文件（第二、三段的全量放不进内存上限）。
const ARM_RECORDS_IN_MEMORY_BEFORE_SPILL: usize = 8 << 20;
static SPILL_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl ArmRecord {
    fn to_bytes(self) -> [u8; ARM_RECORD_BYTES] {
        let mut bytes = [0u8; ARM_RECORD_BYTES];
        bytes[..16].copy_from_slice(&self.key.0.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.ordinal.to_le_bytes());
        bytes[24..40].copy_from_slice(&self.outcome.0.to_le_bytes());
        bytes[40..44].copy_from_slice(&self.elements.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8; ARM_RECORD_BYTES]) -> Self {
        Self {
            key: Digest128(u128::from_le_bytes(
                bytes[..16].try_into().expect("16 字节"),
            )),
            ordinal: u64::from_le_bytes(bytes[16..24].try_into().expect("8 字节")),
            outcome: Digest128(u128::from_le_bytes(
                bytes[24..40].try_into().expect("16 字节"),
            )),
            elements: u32::from_le_bytes(bytes[40..44].try_into().expect("4 字节")),
        }
    }
}

/// 一条臂的记录：内存里一段，外加若干段排好序的溢出文件（放在 `TMPDIR` 下，读完即删）。
struct ArmRecordStore {
    memory: Vec<ArmRecord>,
    runs: Vec<std::path::PathBuf>,
    records_in_memory_before_spill: usize,
}

impl ArmRecordStore {
    fn new(records_in_memory_before_spill: usize) -> Self {
        Self {
            memory: Vec::new(),
            runs: Vec::new(),
            records_in_memory_before_spill,
        }
    }

    fn push(&mut self, record: ArmRecord) {
        self.memory.push(record);
        if self.memory.len() >= self.records_in_memory_before_spill {
            self.spill();
        }
    }

    fn spill(&mut self) {
        use std::io::Write;
        self.memory.sort_unstable();
        let path = std::env::temp_dir().join(format!(
            "e161-arm-records-{}-{}.bin",
            std::process::id(),
            SPILL_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let file = std::fs::File::create_new(&path).expect("建溢出文件");
        let mut writer = std::io::BufWriter::new(file);
        for record in self.memory.drain(..) {
            writer.write_all(&record.to_bytes()).expect("写溢出文件");
        }
        writer.flush().expect("写溢出文件");
        self.runs.push(path);
    }

    fn absorb(&mut self, mut other: ArmRecordStore) {
        self.runs.append(&mut other.runs);
        for record in std::mem::take(&mut other.memory) {
            self.push(record);
        }
    }

    /// 按（键、序号）从小到大交给 `visit`；溢出文件读完就删。
    fn visit_sorted(mut self, visit: &mut dyn FnMut(ArmRecord)) {
        use std::io::Read;
        if self.runs.is_empty() {
            self.memory.sort_unstable();
            for record in self.memory.drain(..) {
                visit(record);
            }
            return;
        }
        if !self.memory.is_empty() {
            self.spill();
        }
        let mut readers: Vec<std::io::BufReader<std::fs::File>> = self
            .runs
            .iter()
            .map(|path| std::io::BufReader::new(std::fs::File::open(path).expect("开溢出文件")))
            .collect();
        let read_next = |reader: &mut std::io::BufReader<std::fs::File>| -> Option<ArmRecord> {
            let mut bytes = [0u8; ARM_RECORD_BYTES];
            match reader.read_exact(&mut bytes) {
                Ok(()) => Some(ArmRecord::from_bytes(&bytes)),
                Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => None,
                Err(error) => panic!("读溢出文件：{error}"),
            }
        };
        let mut heap = std::collections::BinaryHeap::new();
        for (run_index, reader) in readers.iter_mut().enumerate() {
            if let Some(record) = read_next(reader) {
                heap.push(std::cmp::Reverse((record, run_index)));
            }
        }
        while let Some(std::cmp::Reverse((record, run_index))) = heap.pop() {
            visit(record);
            if let Some(next) = read_next(&mut readers[run_index]) {
                heap.push(std::cmp::Reverse((next, run_index)));
            }
        }
        for path in self.runs.drain(..) {
            std::fs::remove_file(&path).expect("删溢出文件");
        }
    }
}

impl Drop for ArmRecordStore {
    fn drop(&mut self) {
        for path in &self.runs {
            let _removed = std::fs::remove_file(path);
        }
    }
}

// ───────────────────────────── 一格的累计与多线程跑 ─────────────────────────────

/// 一段上的计时累计（G3 的加权估计按段取平均）。
#[derive(Clone, Copy, Debug, Default)]
struct TimingSums {
    states: u64,
    state_nanoseconds: u128,
    zone_nanoseconds: [u128; 6],
    unit_events_replay_nanoseconds: u128,
    walk_checker_nanoseconds: u128,
    states_shorter_than_their_segments: u64,
    states_replaying_longer_than_the_checker: u64,
}

impl TimingSums {
    fn absorb_state(&mut self, timings: &StateTimings) {
        self.states += 1;
        self.state_nanoseconds += timings.state_nanoseconds;
        for (slot, zone) in TIMING_ZONES.iter().enumerate() {
            self.zone_nanoseconds[slot] += timings.zone_nanoseconds(*zone);
        }
        self.unit_events_replay_nanoseconds += timings.unit_events_replay_nanoseconds;
        self.walk_checker_nanoseconds += timings.walk_checker_nanoseconds;
        if timings.state_nanoseconds < timings.five_segments_nanoseconds() {
            self.states_shorter_than_their_segments += 1;
        }
        if timings.unit_positions_replay_nanoseconds > timings.pool_checker_nanoseconds {
            self.states_replaying_longer_than_the_checker += 1;
        }
    }

    fn absorb(&mut self, other: &TimingSums) {
        self.states += other.states;
        self.state_nanoseconds += other.state_nanoseconds;
        for (mine, theirs) in self.zone_nanoseconds.iter_mut().zip(other.zone_nanoseconds) {
            *mine += theirs;
        }
        self.unit_events_replay_nanoseconds += other.unit_events_replay_nanoseconds;
        self.walk_checker_nanoseconds += other.walk_checker_nanoseconds;
        self.states_shorter_than_their_segments += other.states_shorter_than_their_segments;
        self.states_replaying_longer_than_the_checker +=
            other.states_replaying_longer_than_the_checker;
    }
}

/// 把 `from` 的 S4 那几项加进 `into`。
fn add_tally(into: &mut Layer0Tally, from: &Layer0Tally) {
    into.states += from.states;
    into.violations += from.violations;
    into.ignored_violations += from.ignored_violations;
    into.root_persisted_states += from.root_persisted_states;
    into.no_file_states += from.no_file_states;
    into.file_read_states += from.file_read_states;
    into.failed_states += from.failed_states;
    into.journal_differing_states += from.journal_differing_states;
    into.verification_ran_states += from.verification_ran_states;
    into.verification_failed_states += from.verification_failed_states;
    into.record_root_without_record += from.record_root_without_record;
    into.record_claimed_state_missing_unit += from.record_claimed_state_missing_unit;
    for (invariant, count) in &from.checker_evaluated_states {
        *into.checker_evaluated_states.entry(invariant).or_insert(0) += count;
    }
    for (invariant, count) in &from.checker_violated_states {
        *into.checker_violated_states.entry(invariant).or_insert(0) += count;
    }
    for (invariant, count) in &from.checker_not_applicable_states {
        *into
            .checker_not_applicable_states
            .entry(invariant)
            .or_insert(0) += count;
    }
}

/// S4 比的那几项拼成一段文本（两边各拼一份逐字比）。
fn tally_text(tally: &Layer0Tally) -> String {
    format!(
        "states={} violations={} ignored_violations={} root_persisted={} no_file={} file_read={} failed={} journal_differing={} verification_ran={} verification_failed={} record_root_without_record={} record_claimed_state_missing_unit={} checker=[{}]",
        tally.states,
        tally.violations,
        tally.ignored_violations,
        tally.root_persisted_states,
        tally.no_file_states,
        tally.file_read_states,
        tally.failed_states,
        tally.journal_differing_states,
        tally.verification_ran_states,
        tally.verification_failed_states,
        tally.record_root_without_record,
        tally.record_claimed_state_missing_unit,
        tally.checker_counts_by_invariant()
    )
}

/// 一格（一条流上的一个取样域）跑完的全部累计。各项都是可交换的加法或取最小，与线程数、片的完成次序无关。
struct CellTotals {
    tally: Layer0Tally,
    tables: ContentTables,
    timing_by_segment: BTreeMap<usize, TimingSums>,
    arm_stores: Vec<ArmRecordStore>,
    whole_unit_positions: u64,
    whole_unit_events: u64,
    header_scan_positions: u64,
    header_scan_events: u64,
    ring_record_positions: u64,
    ring_record_events: u64,
    states_without_whole_unit_reads: u64,
    consult_report_differs: u64,
    ignore_report_differs: u64,
    verdicts_differ: u64,
    walk_reads_outside_normal_reads: u64,
    control: Vec<ControlStateData>,
}

impl CellTotals {
    fn new() -> Self {
        Self {
            tally: Layer0Tally::default(),
            tables: ContentTables::default(),
            timing_by_segment: BTreeMap::new(),
            arm_stores: ARMS
                .iter()
                .map(|_arm| ArmRecordStore::new(ARM_RECORDS_IN_MEMORY_BEFORE_SPILL))
                .collect(),
            whole_unit_positions: 0,
            whole_unit_events: 0,
            header_scan_positions: 0,
            header_scan_events: 0,
            ring_record_positions: 0,
            ring_record_events: 0,
            states_without_whole_unit_reads: 0,
            consult_report_differs: 0,
            ignore_report_differs: 0,
            verdicts_differ: 0,
            walk_reads_outside_normal_reads: 0,
            control: Vec::new(),
        }
    }

    fn absorb_state(&mut self, result: StateResult) {
        self.timing_by_segment
            .entry(result.segment)
            .or_default()
            .absorb_state(&result.timings);
        for (store, record) in self.arm_stores.iter_mut().zip(result.arm_records) {
            store.push(record);
        }
        self.whole_unit_positions += result.whole_unit_positions;
        self.whole_unit_events += result.whole_unit_events;
        self.header_scan_positions += result.header_scan_positions;
        self.header_scan_events += result.header_scan_events;
        self.ring_record_positions += result.ring_record_positions;
        self.ring_record_events += result.ring_record_events;
        if result.whole_unit_positions == 0 {
            self.states_without_whole_unit_reads += 1;
        }
        self.consult_report_differs += u64::from(result.consult_report_differs);
        self.ignore_report_differs += u64::from(result.ignore_report_differs);
        self.verdicts_differ += u64::from(result.verdicts_differ);
        self.walk_reads_outside_normal_reads += u64::from(result.walk_reads_outside_normal_reads);
        if let Some(control) = result.control {
            self.control.push(control);
        }
    }

    fn absorb(&mut self, other: CellTotals) {
        add_tally(&mut self.tally, &other.tally);
        self.tables.absorb(other.tables);
        for (segment, sums) in &other.timing_by_segment {
            self.timing_by_segment
                .entry(*segment)
                .or_default()
                .absorb(sums);
        }
        for (mine, theirs) in self.arm_stores.iter_mut().zip(other.arm_stores) {
            mine.absorb(theirs);
        }
        self.whole_unit_positions += other.whole_unit_positions;
        self.whole_unit_events += other.whole_unit_events;
        self.header_scan_positions += other.header_scan_positions;
        self.header_scan_events += other.header_scan_events;
        self.ring_record_positions += other.ring_record_positions;
        self.ring_record_events += other.ring_record_events;
        self.states_without_whole_unit_reads += other.states_without_whole_unit_reads;
        self.consult_report_differs += other.consult_report_differs;
        self.ignore_report_differs += other.ignore_report_differs;
        self.verdicts_differ += other.verdicts_differ;
        self.walk_reads_outside_normal_reads += other.walk_reads_outside_normal_reads;
        self.control.extend(other.control);
        self.control.sort_by_key(|control| control.ordinal);
    }

    fn states(&self) -> u64 {
        self.tally.states
    }
}

/// 线程数：`E161_THREADS`，没设取 `available_parallelism`（5.6）。
fn threads_from_environment() -> usize {
    match std::env::var(THREADS_ENVIRONMENT_VARIABLE) {
        Ok(text) => text
            .parse::<usize>()
            .ok()
            .filter(|threads| *threads > 0)
            .unwrap_or_else(|| panic!("{THREADS_ENVIRONMENT_VARIABLE} 要是正整数，读到 {text:?}")),
        Err(_not_set) => {
            std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
        }
    }
}

/// 一格要跑的状态：序号（按计划的次序）与要不要留对照数据。
struct CellRun<'run> {
    label: String,
    context: &'run EvaluationContext<'run>,
    plan: &'run LocalPlan<'run>,
    ordinals: &'run [u64],
    control_ordinals: &'run BTreeSet<u64>,
    busy_zone: Option<TimingZone>,
    threads: usize,
    depth: EvaluationDepth,
}

/// 一格里每个状态做到哪一步：停机条款 S4 只要照跑那一遍的计数；产物要 5.1 的四件事。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EvaluationDepth {
    BaselineOnly,
    FourSteps,
}

/// 多线程跑一格：按片领活，每个线程一份累计，最后逐份并起来（并法可交换，结果与线程数无关）。
fn run_cell(run: &CellRun<'_>) -> CellTotals {
    let next_slice = AtomicUsize::new(0);
    let slices: Vec<&[u64]> = run.ordinals.chunks(STATES_PER_SLICE).collect();
    let finished_slices = AtomicUsize::new(0);
    let started = Instant::now();
    let merged = Mutex::new(CellTotals::new());
    std::thread::scope(|scope| {
        for _worker in 0..run.threads {
            scope.spawn(|| {
                let tables = RefCell::new(ContentTables::default());
                let mut totals = CellTotals::new();
                loop {
                    let slice_index = next_slice.fetch_add(1, Ordering::Relaxed);
                    let Some(slice) = slices.get(slice_index) else {
                        break;
                    };
                    for ordinal in *slice {
                        if run.depth == EvaluationDepth::BaselineOnly {
                            let image = CrashImage {
                                base: &run.context.stream.base,
                                writes: &run.context.table.writes,
                                persisted: run.plan.persisted_of(*ordinal),
                            };
                            let mut timings = StateTimings::default();
                            let _reports = evaluate_baseline(
                                run.context,
                                &image,
                                None,
                                &mut timings,
                                &mut totals.tally,
                            );
                            continue;
                        }
                        let result = evaluate_state(
                            run.context,
                            *ordinal,
                            run.plan.segment_of(*ordinal),
                            run.plan.persisted_of(*ordinal),
                            run.busy_zone,
                            run.control_ordinals.contains(ordinal),
                            &tables,
                            &mut totals.tally,
                        );
                        totals.absorb_state(result);
                    }
                    let finished = finished_slices.fetch_add(1, Ordering::Relaxed) + 1;
                    if finished.is_multiple_of(256) || finished == slices.len() {
                        eprintln!(
                            "progress cell={} slices={finished}/{} elapsed_seconds={}",
                            run.label,
                            slices.len(),
                            started.elapsed().as_secs()
                        );
                    }
                }
                totals.tables = tables.into_inner();
                merged.lock().expect("并累计的锁").absorb(totals);
            });
        }
    });
    merged.into_inner().expect("并累计的锁")
}

// ───────────────────────────── 取样域（5.5） ─────────────────────────────

/// 这条流的取样段（第一条流全部；第二条流段 9–22）。
fn is_sampled_segment(stream: StreamName, segment_index: usize) -> bool {
    match stream {
        StreamName::First => true,
        StreamName::Second => (SECOND_STREAM_FIRST_SAMPLED_SEGMENT
            ..=SECOND_STREAM_LAST_SAMPLED_SEGMENT)
            .contains(&segment_index),
    }
}

/// 全部持久那一个在不在域里：第一条流在（D2 = 150994980 含它），第二条流不在（段 23 起不取）。
fn includes_every_write_persisted(stream: StreamName) -> bool {
    match stream {
        StreamName::First => true,
        StreamName::Second => false,
    }
}

fn domain_expansions(stream: &PreparedStream, expansion: LocalExpansion) -> Vec<LocalExpansion> {
    (0..stream.segments.len())
        .map(|segment_index| {
            if is_sampled_segment(stream.name, segment_index) {
                expansion
            } else {
                LocalExpansion::Skipped
            }
        })
        .collect()
}

/// 26 写段（D1-stride 取步长的那两段）。
const STRIDED_SEGMENT_WRITES: usize = 26;
/// 对照样本里「大段」的写数下限（18 写段与 26 写段，5.5）。
const CONTROL_SAMPLE_LARGE_SEGMENT_WRITES: usize = 18;

/// D1-small：取样段里写数少于 26 的每一段的全部状态（第一条流再加全部持久那一个）；按全量计划的序号。
fn small_domain_ordinals(stream: &PreparedStream, plan: &LocalPlan<'_>) -> Vec<u64> {
    let mut ordinals = Vec::new();
    for (segment_index, segment) in stream.segments.iter().enumerate() {
        if is_sampled_segment(stream.name, segment_index) && segment.len() < STRIDED_SEGMENT_WRITES
        {
            let first = plan.first_ordinal_by_segment[segment_index];
            ordinals.extend(first..first + plan.count_by_segment[segment_index]);
        }
    }
    if includes_every_write_persisted(stream.name) {
        ordinals.push(plan.expanded_state_count);
    }
    ordinals
}

/// D1-stride：26 写段里段内序号 ≡ 0 (mod 2311) 的状态。
fn stride_domain_ordinals(stream: &PreparedStream, plan: &LocalPlan<'_>) -> Vec<u64> {
    let mut ordinals = Vec::new();
    for (segment_index, segment) in stream.segments.iter().enumerate() {
        if is_sampled_segment(stream.name, segment_index) && segment.len() == STRIDED_SEGMENT_WRITES
        {
            let first = plan.first_ordinal_by_segment[segment_index];
            let count = plan.count_by_segment[segment_index];
            ordinals.extend(
                (0..count)
                    .step_by(usize::try_from(STRIDE_STEP).expect("步长"))
                    .map(|within| first + within),
            );
        }
    }
    ordinals
}

/// 全部状态（D1-quick 用：整个甲二域，完整枚举）。
fn every_ordinal(stream: &PreparedStream, plan: &LocalPlan<'_>) -> Vec<u64> {
    let last = plan.expanded_state_count + u64::from(includes_every_write_persisted(stream.name));
    (0..last).collect()
}

/// 对照样本（5.5）：D1 里每一段，18 写段与 26 写段取序号最小的 4096 个，其余段取全部；第一条流的全部持久那一个也在。
fn control_ordinals(
    stream: &PreparedStream,
    plan: &LocalPlan<'_>,
    domain_ordinals: &[u64],
) -> BTreeSet<u64> {
    let mut by_segment: BTreeMap<usize, Vec<u64>> = BTreeMap::new();
    for ordinal in domain_ordinals {
        by_segment
            .entry(plan.segment_of(*ordinal))
            .or_default()
            .push(*ordinal);
    }
    let mut chosen = BTreeSet::new();
    for (segment_index, mut ordinals) in by_segment {
        ordinals.sort_unstable();
        let is_large = stream
            .segments
            .get(segment_index)
            .is_some_and(|segment| segment.len() >= CONTROL_SAMPLE_LARGE_SEGMENT_WRITES);
        let taken = if is_large {
            CONTROL_SAMPLE_PER_LARGE_SEGMENT.min(ordinals.len())
        } else {
            ordinals.len()
        };
        chosen.extend(ordinals[..taken].iter().copied());
    }
    chosen
}

// ───────────────────────────── 停机条款 S3、S4 与锚点 A1 ─────────────────────────────

/// A1：一段的三方状态数（装置计划逐个数出来的、crates 只展开那一段算的、公式算的）。
struct SegmentCountCheck {
    segment_index: usize,
    writes: usize,
    in_place_overwrites: u64,
    counted_by_the_plan: u64,
    counted_by_the_crates: u64,
    counted_by_the_formula: u64,
}

/// 逐个数：只展开 `segment_index` 那一段的计划里，每个序号的持久集合真生成出来，数不同的有几个。取样段里一律逐个数（段内这一段的写、
/// 它们的撕裂镜像与重放至多 32 个位置时用位图，否则用集合）；取样段之外（第二条流段 23 起，一段至多 3² · 2²⁸ − 1 个）按计划的组合数报。
/// 交回（数出来的个数，是不是逐个数的）。
fn count_distinct_persisted_sets(
    stream: &PreparedStream,
    table: &TornWriteTable,
    plan: &LocalPlan<'_>,
    segment_index: usize,
    enumeration_limit: u64,
) -> (u64, bool) {
    if !is_sampled_segment(stream.name, segment_index)
        || plan.expanded_state_count > enumeration_limit
    {
        return (plan.expanded_state_count, false);
    }
    let mut positions: Vec<usize> = stream.segments[segment_index].clone();
    for write_index in &stream.segments[segment_index] {
        if let Some((torn_index, replays)) = table.torn_image_by_write.get(write_index) {
            positions.push(*torn_index);
            positions.extend(
                replays
                    .iter()
                    .map(|(replay_index, _replayed)| *replay_index),
            );
        }
    }
    let code_of = |persisted: &[bool]| -> u64 {
        positions
            .iter()
            .enumerate()
            .filter(|(_bit, position)| persisted[**position])
            .map(|(bit, _position)| 1u64 << bit)
            .sum()
    };
    if positions.len() <= 32 {
        let mut seen = vec![0u64; (1usize << positions.len()).div_ceil(64)];
        let mut distinct = 0u64;
        for ordinal in 0..plan.expanded_state_count {
            let code = usize::try_from(code_of(&plan.persisted_of(ordinal))).expect("至多 32 位");
            let (word, bit) = (code / 64, code % 64);
            if seen[word] & (1 << bit) == 0 {
                seen[word] |= 1 << bit;
                distinct += 1;
            }
        }
        (distinct, true)
    } else {
        let distinct: BTreeSet<Vec<bool>> = (0..plan.expanded_state_count)
            .map(|ordinal| plan.persisted_of(ordinal))
            .collect();
        (u64::try_from(distinct.len()).expect("个数"), true)
    }
}

/// A1：逐段只展开那一段，装置计划逐个数（取样段里持久集合真生成、不同即数）、crates 算、公式算。
fn segment_count_checks(stream: &PreparedStream, table: &TornWriteTable) -> Vec<SegmentCountCheck> {
    segment_count_checks_up_to(stream, table, u64::MAX)
}

/// 同上，一段的状态数超过 `enumeration_limit` 时不逐个数、按组合数报（单测在调试构建下用小的上限）。
fn segment_count_checks_up_to(
    stream: &PreparedStream,
    table: &TornWriteTable,
    enumeration_limit: u64,
) -> Vec<SegmentCountCheck> {
    (0..stream.segments.len())
        .map(|segment_index| {
            let expansions: Vec<LocalExpansion> = (0..stream.segments.len())
                .map(|index| {
                    if index == segment_index {
                        LocalExpansion::EveryCombination
                    } else {
                        LocalExpansion::Skipped
                    }
                })
                .collect();
            let plan = LocalPlan::new(stream, table, expansions);
            let (counted_by_the_plan, _enumerated) = count_distinct_persisted_sets(
                stream,
                table,
                &plan,
                segment_index,
                enumeration_limit,
            );
            let crates_count =
                singlefs_harness::crash::layer0_state_count_with_torn_in_place_overwrites(
                    &stream.base,
                    &stream.writes,
                    &stream.segments,
                    &|index, _segment| {
                        if index == segment_index {
                            Layer0SegmentExpansion::EveryProperSubset
                        } else {
                            Layer0SegmentExpansion::NotExpanded
                        }
                    },
                ) - 1;
            let segment = &stream.segments[segment_index];
            let in_place_overwrites = u64::try_from(
                segment
                    .iter()
                    .filter(|write_index| table.is_in_place_overwrite[**write_index])
                    .count(),
            )
            .expect("个数");
            SegmentCountCheck {
                segment_index,
                writes: segment.len(),
                in_place_overwrites,
                counted_by_the_plan,
                counted_by_the_crates: crates_count,
                counted_by_the_formula: formula_segment_state_count(
                    u64::try_from(segment.len()).expect("写数"),
                    in_place_overwrites,
                ),
            }
        })
        .collect()
}

/// S3 与 S4 在一个取样域上的结果：crates 枚举交给观察者的写表与持久集合逐个与装置比，crates 的计数交回来给 S4 比。
struct CratesComparison {
    observed_states: u64,
    compared_states: u64,
    writes_table_differs: bool,
    persisted_mismatches: u64,
    crates_tally: Layer0Tally,
}

/// 在 `ordinals`（装置计划的序号，按次序）那个域上跑一遍 crates 的枚举（只展开 `expansion_of` 给的那几段），逐状态核 S3，交回 crates 的计数。
/// crates 最后多出的全部持久那一个（域里不含它时）不比。
fn compare_with_the_crates(
    stream: &PreparedStream,
    table: &TornWriteTable,
    plan: &LocalPlan<'_>,
    ordinals: &[u64],
    expansion_of: &dyn Fn(usize) -> Layer0SegmentExpansion,
) -> CratesComparison {
    let mut observed_states = 0u64;
    let mut compared_states = 0u64;
    let mut writes_table_differs = false;
    let mut writes_table_checked: Option<*const RetainedWrite> = None;
    let mut persisted_mismatches = 0u64;
    let mut observe =
        |image: &CrashImage<'_>,
         _report: &RecoveryReport,
         _counts: &mut singlefs_harness::crash::Layer0ObserverCounts| {
            let position = usize::try_from(observed_states).expect("序号");
            observed_states += 1;
            let Some(ordinal) = ordinals.get(position) else {
                return;
            };
            compared_states += 1;
            match writes_table_checked {
                Some(pointer) if std::ptr::eq(pointer, image.writes.as_ptr()) => {}
                Some(_) | None => {
                    if image.writes != table.writes.as_slice() {
                        writes_table_differs = true;
                    }
                    writes_table_checked = Some(image.writes.as_ptr());
                }
            }
            if image.persisted != plan.persisted_of(*ordinal) {
                persisted_mismatches += 1;
            }
        };
    let crates_tally = singlefs_harness::crash::enumerate_layer0_in_state_slices(
        &stream.base,
        &stream.writes,
        &stream.segments,
        stream.judged_root_index,
        &stream.versions,
        &|segment_index, _segment| expansion_of(segment_index),
        singlefs_harness::crash::Layer0Parallelism::from_environment(),
        Some(&mut observe),
        &singlefs_harness::layer0_progress::Layer0Resume::NoProgressFile,
    );
    CratesComparison {
        observed_states,
        compared_states,
        writes_table_differs,
        persisted_mismatches,
        crates_tally,
    }
}

/// 装置这一格的计数加上（第二条流 S4 要比的）crates 多跑的全部持久那一个的照跑计数。
fn tally_with_every_write_persisted(
    context: &EvaluationContext<'_>,
    tally: &Layer0Tally,
) -> Layer0Tally {
    let mut combined = Layer0Tally::default();
    add_tally(&mut combined, tally);
    if !includes_every_write_persisted(context.stream.name) {
        let every_write = LocalPlan::new(
            context.stream,
            context.table,
            vec![LocalExpansion::Skipped; context.stream.segments.len()],
        );
        let image = CrashImage {
            base: &context.stream.base,
            writes: &context.table.writes,
            persisted: every_write.persisted_of(0),
        };
        let mut timings = StateTimings::default();
        let _reports = evaluate_baseline(context, &image, None, &mut timings, &mut combined);
    }
    combined
}

// ───────────────────────────── 计数：复用臂、轨迹（第六节、8.1） ─────────────────────────────

/// 轨迹的区间（8.1）：每一段的末尾一个检查点，段内每 2²⁰ 个状态一个；D1-stride 的段按取样状态每 2¹² 个一个。
struct TrajectoryIntervals {
    /// 每个区间的第一个序号，递增。
    starts: Vec<u64>,
}

impl TrajectoryIntervals {
    fn of(
        plan: &LocalPlan<'_>,
        sorted_ordinals: &[u64],
        strided_segments: &BTreeSet<usize>,
    ) -> Self {
        let mut starts = Vec::new();
        let mut current_segment = None;
        let mut states_in_interval = 0u64;
        for ordinal in sorted_ordinals {
            let segment = plan.segment_of(*ordinal);
            let per_checkpoint = if strided_segments.contains(&segment) {
                TRAJECTORY_STRIDE_SAMPLES_PER_CHECKPOINT
            } else {
                TRAJECTORY_STATES_PER_CHECKPOINT
            };
            if current_segment != Some(segment) || states_in_interval == per_checkpoint {
                starts.push(*ordinal);
                current_segment = Some(segment);
                states_in_interval = 0;
            }
            states_in_interval += 1;
        }
        Self { starts }
    }

    fn interval_of(&self, ordinal: u64) -> usize {
        self.starts
            .partition_point(|start| *start <= ordinal)
            .saturating_sub(1)
    }

    fn count(&self) -> usize {
        self.starts.len()
    }
}

/// 一个量的轨迹（8.1）：每个区间的增量。
struct Trajectory {
    increments: Vec<u128>,
}

impl Trajectory {
    fn new(intervals: &TrajectoryIntervals) -> Self {
        Self {
            increments: vec![0; intervals.count().max(1)],
        }
    }

    fn add(&mut self, interval: usize, amount: u128) {
        self.increments[interval] += amount;
    }

    /// `名字_peak_after_cold_start=… 名字_positive_intervals=… 名字_final=… 名字_last_four=[…]`。
    fn fields(&self, name: &str) -> String {
        let peak_after_cold_start = self.increments.iter().skip(1).copied().max().unwrap_or(0);
        let positive = self
            .increments
            .iter()
            .filter(|increment| **increment > 0)
            .count();
        let total: u128 = self.increments.iter().sum();
        let last_four: Vec<String> = self
            .increments
            .iter()
            .rev()
            .take(4)
            .rev()
            .map(u128::to_string)
            .collect();
        format!(
            "{name}_peak_after_cold_start={peak_after_cold_start} {name}_positive_intervals={positive}/{} {name}_final={total} {name}_last_four=[{}]",
            self.increments.len(),
            last_four.join(",")
        )
    }
}

/// 一条复用臂在一格上的计数（第六节 Q2a–Q2c、Q4a–Q4c，G5 P3）。
struct ArmCount {
    states: u64,
    distinct_keys: u64,
    hits: u64,
    inconsistent: u64,
    /// 每个不同键第一条记录的大小之和（G5 P3 的元素或调用数）。
    distinct_key_elements: u64,
    new_keys: Trajectory,
    inconsistencies: Trajectory,
    /// PC2b / PC4b：键换成常数时的不一致数（与序号最小那个状态结局不同的状态数）。
    constant_key_inconsistent: u64,
    distinct_outcomes: u64,
    /// 结局不同的两个状态（PC4b 找不到时挑一对强行设同键用）：（序号，序号）。
    differing_pair: Option<(u64, u64)>,
}

/// 按（键、序号）顺次走一遍：同键里序号最小那个是参照，其余是命中，结局与参照不同即不一致（Q2a / Q4a）。
fn count_arm(
    store: ArmRecordStore,
    intervals: &TrajectoryIntervals,
    copy_into: Option<&mut ArmRecordStore>,
) -> ArmCount {
    let mut count = ArmCount {
        states: 0,
        distinct_keys: 0,
        hits: 0,
        inconsistent: 0,
        distinct_key_elements: 0,
        new_keys: Trajectory::new(intervals),
        inconsistencies: Trajectory::new(intervals),
        constant_key_inconsistent: 0,
        distinct_outcomes: 0,
        differing_pair: None,
    };
    let mut current: Option<(Digest128, Digest128)> = None;
    let mut outcome_counts: HashMap<Digest128, (u64, u64)> = HashMap::new();
    let mut minimum: Option<(u64, Digest128)> = None;
    let mut copy_into = copy_into;
    store.visit_sorted(&mut |record| {
        if let Some(target) = copy_into.as_deref_mut() {
            target.push(record);
        }
        count.states += 1;
        let entry = outcome_counts
            .entry(record.outcome)
            .or_insert((0, record.ordinal));
        entry.0 += 1;
        entry.1 = entry.1.min(record.ordinal);
        if minimum.is_none_or(|(ordinal, _)| record.ordinal < ordinal) {
            minimum = Some((record.ordinal, record.outcome));
        }
        match current {
            Some((key, reference)) if key == record.key => {
                count.hits += 1;
                if record.outcome != reference {
                    count.inconsistent += 1;
                    count
                        .inconsistencies
                        .add(intervals.interval_of(record.ordinal), 1);
                }
            }
            Some(_) | None => {
                current = Some((record.key, record.outcome));
                count.distinct_keys += 1;
                count.distinct_key_elements += u64::from(record.elements);
                count.new_keys.add(intervals.interval_of(record.ordinal), 1);
            }
        }
    });
    if let Some((_ordinal, outcome)) = minimum {
        let same = outcome_counts
            .get(&outcome)
            .map_or(0, |(states, _)| *states);
        count.constant_key_inconsistent = count.states - same;
    }
    count.distinct_outcomes = u64::try_from(outcome_counts.len()).expect("个数");
    let mut firsts: Vec<u64> = outcome_counts.values().map(|(_, first)| *first).collect();
    firsts.sort_unstable();
    if let [first, second, ..] = firsts.as_slice() {
        count.differing_pair = Some((*first, *second));
    }
    count
}

/// 比值的文本：分子 / 分母，六位小数（判定用整数比较，不用这个文本）。
fn ratio_text(numerator: u128, denominator: u128) -> String {
    if denominator == 0 {
        return "undefined".to_string();
    }
    let scaled = numerator * 1_000_000 / denominator;
    format!("{}.{:06}", scaled / 1_000_000, scaled % 1_000_000)
}

/// 「比值 ≥ 0.5」的整数判定（登记第一节：接近 1 / 接近状态数 = 比值 ≥ 0.5）。
fn reaches_half(numerator: u128, denominator: u128) -> bool {
    ratio_reaches(
        numerator,
        denominator,
        u128::from(RATIO_THRESHOLD_NUMERATOR),
        u128::from(RATIO_THRESHOLD_DENOMINATOR),
    )
}

/// 比值 numerator / denominator ≥ threshold_numerator / threshold_denominator（判别力自证把门槛挪到两点之间时用同一个判定函数）。
fn ratio_reaches(
    numerator: u128,
    denominator: u128,
    threshold_numerator: u128,
    threshold_denominator: u128,
) -> bool {
    numerator * threshold_denominator >= denominator * threshold_numerator
}

// ───────────────────────────── 一格的结果行（G1–G5、S5） ─────────────────────────────

/// 一格报完之后留给判定、几何敏感性与判别力自证用的数。
#[derive(Clone, Debug)]
struct CellSummary {
    label: String,
    states: u64,
    distinct_unit_contents: u64,
    per_state_unit_checks: u64,
    arm_distinct_keys: [u64; 6],
    arm_inconsistent: [u64; 6],
    arm_hits: [u64; 6],
    arm_constant_key_inconsistent: [u64; 6],
    arm_distinct_outcomes: [u64; 6],
    arm_differing_pair: [Option<(u64, u64)>; 6],
    arm_distinct_key_elements: [u64; 6],
    state_nanoseconds: u128,
    zone_nanoseconds: [u128; 6],
    unit_events_replay_nanoseconds: u128,
    walk_checker_nanoseconds: u128,
    content_bytes_by_category: [u128; 4],
    unit_contents_seen: BTreeSet<Digest128>,
    consult_report_differs: u64,
    ignore_report_differs: u64,
    verdicts_differ: u64,
    walk_reads_outside_normal_reads: u64,
    content_collisions: u64,
    states_shorter_than_their_segments: u64,
    states_replaying_longer_than_the_checker: u64,
    /// 结果行里不含计时的部分（V6 逐字比）。
    lines_without_timing: Vec<String>,
}

impl CellSummary {
    fn empty(label: &str, states: u64) -> Self {
        Self {
            label: label.to_string(),
            states,
            distinct_unit_contents: 0,
            per_state_unit_checks: 0,
            arm_distinct_keys: [0; 6],
            arm_inconsistent: [0; 6],
            arm_hits: [0; 6],
            arm_constant_key_inconsistent: [0; 6],
            arm_distinct_outcomes: [0; 6],
            arm_differing_pair: [None; 6],
            arm_distinct_key_elements: [0; 6],
            state_nanoseconds: 0,
            zone_nanoseconds: [0; 6],
            unit_events_replay_nanoseconds: 0,
            walk_checker_nanoseconds: 0,
            content_bytes_by_category: [0; 4],
            unit_contents_seen: BTreeSet::new(),
            consult_report_differs: 0,
            ignore_report_differs: 0,
            verdicts_differ: 0,
            walk_reads_outside_normal_reads: 0,
            content_collisions: 0,
            states_shorter_than_their_segments: 0,
            states_replaying_longer_than_the_checker: 0,
            lines_without_timing: Vec::new(),
        }
    }
}

/// 结果行去掉计时字段（名字以 `_ns`、`_ratio_of_state` 结尾的，与线程数、机器负载有关）。
fn without_timing_fields(line: &str) -> String {
    line.split(' ')
        .filter(|field| {
            let name = field.split('=').next().unwrap_or("");
            !(name.ends_with("_elapsed_ns")
                || name.ends_with("_ratio")
                || name.starts_with("threads"))
        })
        .collect::<Vec<&str>>()
        .join(" ")
}

/// 一格的全部结果行。`stores` 是这一格的六条臂的记录（报完即耗掉）；`union_stores` 给了就把每条记录也抄进去（D1 并格用）。
fn report_cell(
    output: &mut ResultLines,
    label: &str,
    totals: &CellTotals,
    stores: Vec<ArmRecordStore>,
    intervals: &TrajectoryIntervals,
    union_stores: Option<&mut Vec<ArmRecordStore>>,
) -> CellSummary {
    let mut summary = CellSummary::empty(label, totals.states());
    let emit =
        |destination: &mut ResultLines, lines_without_timing: &mut Vec<String>, line: String| {
            lines_without_timing.push(without_timing_fields(&line));
            destination.line(line);
        };
    let mut union_stores = union_stores;
    let mut arm_counts = Vec::new();
    for (slot, store) in stores.into_iter().enumerate() {
        let target = union_stores.as_deref_mut().map(|union| &mut union[slot]);
        arm_counts.push(count_arm(store, intervals, target));
    }
    for (slot, count) in arm_counts.iter().enumerate() {
        summary.arm_distinct_keys[slot] = count.distinct_keys;
        summary.arm_inconsistent[slot] = count.inconsistent;
        summary.arm_hits[slot] = count.hits;
        summary.arm_constant_key_inconsistent[slot] = count.constant_key_inconsistent;
        summary.arm_distinct_outcomes[slot] = count.distinct_outcomes;
        summary.arm_differing_pair[slot] = count.differing_pair;
        summary.arm_distinct_key_elements[slot] = count.distinct_key_elements;
    }
    // G1（Q1a–Q1f）与内容表。
    let unit_table = &totals.tables.by_category[ReadCategory::WholeUnit.slot()];
    let mut unit_trajectory = Trajectory::new(intervals);
    for (digest, entry) in &unit_table.entries {
        if let Some(first) = entry.first_ordinal_checker_normal_pass {
            summary.distinct_unit_contents += 1;
            summary.unit_contents_seen.insert(*digest);
            unit_trajectory.add(intervals.interval_of(first), 1);
        }
    }
    summary.per_state_unit_checks = totals.whole_unit_positions;
    let checker_only_distinct = |category: ReadCategory| {
        totals.tables.by_category[category.slot()]
            .entries
            .values()
            .filter(|entry| entry.first_ordinal_checker_normal_pass.is_some())
            .count()
    };
    let header_distinct = checker_only_distinct(ReadCategory::UnitHeaderScan);
    let ring_distinct = checker_only_distinct(ReadCategory::JournalRing);
    summary.content_collisions = totals.tables.collisions();
    emit(output, &mut summary.lines_without_timing, format!(
        "name=g1 cell={label} states={} distinct_unit_contents={} per_state_unit_checks={} r1={} r1_at_or_above_half={} unit_read_events={} states_without_unit_reads={} header_scan_events={} header_scan_per_state_positions={} header_scan_distinct_contents={header_distinct} ring_record_events={} ring_record_per_state_positions={} ring_record_distinct_contents={ring_distinct} ring_record_r={} content_collisions={} {}",
        summary.states,
        summary.distinct_unit_contents,
        summary.per_state_unit_checks,
        ratio_text(u128::from(summary.distinct_unit_contents), u128::from(summary.per_state_unit_checks)),
        reaches_half(u128::from(summary.distinct_unit_contents), u128::from(summary.per_state_unit_checks)),
        totals.whole_unit_events,
        totals.states_without_whole_unit_reads,
        totals.header_scan_events,
        totals.header_scan_positions,
        totals.ring_record_events,
        totals.ring_record_positions,
        ratio_text(u128::try_from(ring_distinct).expect("个数"), u128::from(totals.ring_record_positions)),
        summary.content_collisions,
        unit_trajectory.fields("q1a"),
    ));
    // G2（两遍各一套 + 成对键）与 G4（三条臂各一套）。
    for arm in ARMS {
        let count = &arm_counts[arm.slot()];
        emit(output, &mut summary.lines_without_timing, format!(
            "name=reuse_arm cell={label} arm={} states={} inconsistent={} hits={} distinct_keys={} keys_over_states={} keys_at_or_above_half={} constant_key_inconsistent={} distinct_outcomes={} {} {}",
            arm.label(),
            count.states,
            count.inconsistent,
            count.hits,
            count.distinct_keys,
            ratio_text(u128::from(count.distinct_keys), u128::from(count.states)),
            reaches_half(u128::from(count.distinct_keys), u128::from(count.states)),
            count.constant_key_inconsistent,
            count.distinct_outcomes,
            count.new_keys.fields("new_keys"),
            count.inconsistencies.fields("inconsistent"),
        ));
    }
    // S5 与 Q4e（S6）。
    summary.consult_report_differs = totals.consult_report_differs;
    summary.ignore_report_differs = totals.ignore_report_differs;
    summary.verdicts_differ = totals.verdicts_differ;
    summary.walk_reads_outside_normal_reads = totals.walk_reads_outside_normal_reads;
    emit(output, &mut summary.lines_without_timing, format!(
        "name=determinism cell={label} states={} recorded_consult_report_differs={} recorded_ignore_report_differs={} recorded_verdicts_differ={} walk_unit_reads_outside_normal_unit_reads={}",
        summary.states,
        totals.consult_report_differs,
        totals.ignore_report_differs,
        totals.verdicts_differ,
        totals.walk_reads_outside_normal_reads,
    ));
    // P1 四类与它的轨迹。
    let mut content_trajectory = Trajectory::new(intervals);
    let mut category_fields = Vec::new();
    for category in READ_CATEGORIES {
        let table = &totals.tables.by_category[category.slot()];
        let mut bytes = 0u128;
        let mut distinct = 0u64;
        for entry in table.entries.values() {
            if let Some(first) = entry.first_ordinal_any_pass {
                let length = u128::try_from(entry.bytes.len()).expect("长度");
                bytes += length;
                distinct += 1;
                content_trajectory.add(intervals.interval_of(first), length);
            }
        }
        summary.content_bytes_by_category[category.slot()] = bytes;
        category_fields.push(format!(
            "{}_bytes={bytes} {}_distinct={distinct}",
            category.label(),
            category.label()
        ));
    }
    emit(
        output,
        &mut summary.lines_without_timing,
        format!(
            "name=g5_contents cell={label} states={} {} p1_bytes={} {}",
            summary.states,
            category_fields.join(" "),
            summary.content_bytes_by_category.iter().sum::<u128>(),
            content_trajectory.fields("p1"),
        ),
    );
    // G3 这一格的总时长与占比（未加权）。
    let mut sums = TimingSums::default();
    for segment_sums in totals.timing_by_segment.values() {
        sums.absorb(segment_sums);
    }
    summary.state_nanoseconds = sums.state_nanoseconds;
    summary.zone_nanoseconds = sums.zone_nanoseconds;
    summary.unit_events_replay_nanoseconds = sums.unit_events_replay_nanoseconds;
    summary.walk_checker_nanoseconds = sums.walk_checker_nanoseconds;
    summary.states_shorter_than_their_segments = sums.states_shorter_than_their_segments;
    summary.states_replaying_longer_than_the_checker =
        sums.states_replaying_longer_than_the_checker;
    output.line(timing_line(label, &sums));
    summary
}

/// G3 的一行（全部是计时字段，不进 V6 的逐字比）：五段与单元级重放的总时长与占 ΣT_state 的比、Q3g、Q3h、Q3i、Q3j、V3 的两个数。
fn timing_line(label: &str, sums: &TimingSums) -> String {
    let share = |nanoseconds: u128| ratio_text(nanoseconds, sums.state_nanoseconds);
    let zone_fields: Vec<String> = TIMING_ZONES
        .iter()
        .enumerate()
        .map(|(slot, zone)| {
            format!(
                "{}_elapsed_ns={} {}_share_ratio={}",
                zone.label(),
                sums.zone_nanoseconds[slot],
                zone.label(),
                share(sums.zone_nanoseconds[slot])
            )
        })
        .collect();
    let checker = sums.zone_nanoseconds[3];
    let unit_positions = sums.zone_nanoseconds[5];
    format!(
        "name=g3 cell={label} states={} state_elapsed_ns={} {} unit_events_replay_elapsed_ns={} unit_events_share_ratio={} walk_and_judge_lower_elapsed_ns={} walk_and_judge_upper_elapsed_ns={} walk_checker_elapsed_ns={} walk_checker_of_checker_share_ratio={} state_shorter_than_its_segments_share_ratio={} replaying_longer_than_the_checker_share_ratio={}",
        sums.states,
        sums.state_nanoseconds,
        zone_fields.join(" "),
        sums.unit_events_replay_nanoseconds,
        share(sums.unit_events_replay_nanoseconds),
        checker.saturating_sub(sums.unit_events_replay_nanoseconds),
        checker.saturating_sub(unit_positions),
        sums.walk_checker_nanoseconds,
        ratio_text(sums.walk_checker_nanoseconds, checker),
        ratio_text(u128::from(sums.states_shorter_than_their_segments), u128::from(sums.states)),
        ratio_text(u128::from(sums.states_replaying_longer_than_the_checker), u128::from(sums.states)),
    )
}

// ───────────────────────────── G5：字节数与外推（第六节 G5、A6） ─────────────────────────────

/// 一种装入在一个容量上的判定（第六节 G5 表下：95% 到 100% 之间记临界）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CapacityVerdict {
    Fits,
    Critical,
    DoesNotFit,
}

impl CapacityVerdict {
    fn of(required: u128, capacity: u128) -> Self {
        if required > capacity {
            Self::DoesNotFit
        } else if required * 100 >= capacity * CRITICAL_BAND_PERCENT {
            Self::Critical
        } else {
            Self::Fits
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Fits => "fits",
            Self::Critical => "critical",
            Self::DoesNotFit => "does_not_fit",
        }
    }
}

/// G5 的四部分（字节）：P1 去重后的内容、P2 每状态子集掩码、P3 读集、P4 集合键（分每状态那部分与每个不同键那部分）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DeviceMemoryParts {
    contents: u128,
    masks: u128,
    read_sets: u128,
    keys_per_state: u128,
    keys_per_distinct_key: u128,
}

impl DeviceMemoryParts {
    fn total(&self) -> u128 {
        self.contents
            + self.masks
            + self.read_sets
            + self.keys_per_state
            + self.keys_per_distinct_key
    }

    /// 五卡切开：P1 每张卡各放一份，其余切开放（第六节 Q5b 的式子）。
    fn five_cards(&self) -> CapacityVerdict {
        let per_card = SMALLEST_SIXTEEN_GIGABYTE_CARD_MEBIBYTES * MEBIBYTE;
        let all_cards = FIVE_CARDS_MEBIBYTES * MEBIBYTE;
        if self.contents > per_card || CARD_COUNT * self.contents > all_cards {
            return CapacityVerdict::DoesNotFit;
        }
        let contents_verdict = CapacityVerdict::of(self.contents, per_card);
        let rest_verdict = CapacityVerdict::of(
            self.total() - self.contents,
            all_cards - CARD_COUNT * self.contents,
        );
        match (contents_verdict, rest_verdict) {
            (CapacityVerdict::DoesNotFit, _) | (_, CapacityVerdict::DoesNotFit) => {
                CapacityVerdict::DoesNotFit
            }
            (CapacityVerdict::Critical, _) | (_, CapacityVerdict::Critical) => {
                CapacityVerdict::Critical
            }
            (CapacityVerdict::Fits, CapacityVerdict::Fits) => CapacityVerdict::Fits,
        }
    }

    /// Q5b：四部分都按 N_ext / N 放大。
    fn linear(&self, from_states: u128, to_states: u128) -> Self {
        let scale = |bytes: u128| bytes * to_states / from_states;
        Self {
            contents: scale(self.contents),
            masks: scale(self.masks),
            read_sets: scale(self.read_sets),
            keys_per_state: scale(self.keys_per_state),
            keys_per_distinct_key: scale(self.keys_per_distinct_key),
        }
    }

    /// Q5c：去重的部分（P1、P3、P4 每个不同键那部分）不涨，每状态的部分按 N_ext / N 放大。
    fn deduplicated_parts_fixed(&self, from_states: u128, to_states: u128) -> Self {
        let scale = |bytes: u128| bytes * to_states / from_states;
        Self {
            contents: self.contents,
            masks: scale(self.masks),
            read_sets: self.read_sets,
            keys_per_state: scale(self.keys_per_state),
            keys_per_distinct_key: self.keys_per_distinct_key,
        }
    }
}

/// 每状态子集掩码的字节数：⌈W / 8⌉。
fn mask_bytes_per_state(write_table_length: usize) -> u128 {
    u128::try_from(write_table_length.div_ceil(8)).expect("掩码字节")
}

/// 一格在一条 K4 臂下的四部分。
fn device_memory_parts(
    summary: &CellSummary,
    write_table_length: usize,
    checker_arm: Arm,
) -> DeviceMemoryParts {
    let states = u128::from(summary.states);
    let read_set_bytes = |arm: Arm| {
        u128::from(summary.arm_distinct_key_elements[arm.slot()])
            * u128::from(arm.bytes_per_element())
    };
    DeviceMemoryParts {
        contents: summary.content_bytes_by_category.iter().sum(),
        masks: mask_bytes_per_state(write_table_length) * states,
        read_sets: read_set_bytes(Arm::RecoveryConsult)
            + read_set_bytes(Arm::RecoveryIgnore)
            + read_set_bytes(checker_arm),
        keys_per_state: states * u128::from(KEYS_STORED_PER_STATE * KEY_BYTES),
        keys_per_distinct_key: u128::from(
            summary.arm_distinct_keys[Arm::RecoveryConsult.slot()]
                + summary.arm_distinct_keys[Arm::RecoveryIgnore.slot()]
                + summary.arm_distinct_keys[checker_arm.slot()],
        ) * u128::from(DISTINCT_KEY_ENTRY_BYTES),
    }
}

/// 一组外推的判定字段：单卡 32 GB、16 GB 那一档单卡、五卡切开。
fn capacity_fields(prefix: &str, parts: &DeviceMemoryParts) -> (String, [CapacityVerdict; 3]) {
    let largest = CapacityVerdict::of(parts.total(), LARGEST_CARD_MEBIBYTES * MEBIBYTE);
    let sixteen = CapacityVerdict::of(
        parts.total(),
        SMALLEST_SIXTEEN_GIGABYTE_CARD_MEBIBYTES * MEBIBYTE,
    );
    let five = parts.five_cards();
    (
        format!(
            "{prefix}_bytes={} {prefix}_largest_card={} {prefix}_sixteen_gigabyte_card={} {prefix}_five_cards={}",
            parts.total(),
            largest.label(),
            sixteen.label(),
            five.label()
        ),
        [largest, sixteen, five],
    )
}

/// G5 的一格一臂：四部分、每状态平均、Q5b / Q5c / Q5d / Q5e（在各判定字段里）、Q5f、Q5g。
fn report_device_memory(
    output: &mut ResultLines,
    summary: &CellSummary,
    write_table_length: usize,
    second_stream_write_table_length: usize,
    checker_arm: Arm,
) -> [[CapacityVerdict; 3]; 2] {
    let parts = device_memory_parts(summary, write_table_length, checker_arm);
    let states = u128::from(summary.states);
    let (linear_fields, linear) =
        capacity_fields("q5b", &parts.linear(states, EXTRAPOLATED_STATES));
    let (fixed_fields, fixed) = capacity_fields(
        "q5c",
        &parts.deduplicated_parts_fixed(states, EXTRAPOLATED_STATES),
    );
    let agreement: Vec<String> = ["largest_card", "sixteen_gigabyte_card", "five_cards"]
        .iter()
        .zip(linear.iter().zip(&fixed))
        .map(|(capacity, (left, right))| format!("q5d_{capacity}_agree={}", left == right))
        .collect();
    let second_stream_parts = DeviceMemoryParts {
        contents: parts.contents,
        masks: mask_bytes_per_state(second_stream_write_table_length) * states,
        read_sets: parts.read_sets,
        keys_per_state: parts.keys_per_state,
        keys_per_distinct_key: parts.keys_per_distinct_key,
    };
    let (full_linear_fields, _) = capacity_fields(
        "q5f_linear",
        &second_stream_parts.linear(states, SECOND_STREAM_FULL_STATES),
    );
    let (full_fixed_fields, _) = capacity_fields(
        "q5f_fixed",
        &second_stream_parts.deduplicated_parts_fixed(states, SECOND_STREAM_FULL_STATES),
    );
    let per_state_of_largest_segment = DeviceMemoryParts {
        contents: parts.contents,
        masks: mask_bytes_per_state(second_stream_write_table_length)
            * SECOND_STREAM_LARGEST_SEGMENT_STATES,
        read_sets: 0,
        keys_per_state: SECOND_STREAM_LARGEST_SEGMENT_STATES
            * u128::from(KEYS_STORED_PER_STATE * KEY_BYTES),
        keys_per_distinct_key: 0,
    };
    let (segment_batch_fields, _) =
        capacity_fields("q5g_largest_segment_batch", &per_state_of_largest_segment);
    output.line(format!(
        "name=g5 cell={} checker_arm={} states={states} w={write_table_length} p1_bytes={} p2_bytes={} p3_bytes={} p4_per_state_bytes={} p4_distinct_key_bytes={} total_bytes={} bytes_per_state={} {linear_fields} {fixed_fields} {} {full_linear_fields} {full_fixed_fields} {segment_batch_fields}",
        summary.label,
        checker_arm.label(),
        parts.contents,
        parts.masks,
        parts.read_sets,
        parts.keys_per_state,
        parts.keys_per_distinct_key,
        parts.total(),
        ratio_text(parts.total(), states),
        agreement.join(" "),
    ));
    [linear, fixed]
}

// ───────────────────────────── PC2c、PC4c：去掉一个位置再算键（5.3） ─────────────────────────────

/// 对照样本上一条臂的一个状态：原来的键、结局、按位置可去掉的元素（读集按次序，集合按位置排序）。
struct ControlArmState {
    key: Digest128,
    outcome: Digest128,
    /// 集合臂的所选根；读集臂不用。
    root: Option<Digest128>,
    /// （位置摘要，元素摘要）。
    elements: Vec<(Digest128, Digest128)>,
    /// 这个状态出现过的位置，排好序去重（判「含不含 p」用）。
    positions: Vec<Digest128>,
}

fn control_arm_states(
    control: &[ControlStateData],
    arm: Arm,
) -> (
    Vec<ControlArmState>,
    BTreeMap<Digest128, PositionDescription>,
) {
    let mut descriptions = BTreeMap::new();
    let states = control
        .iter()
        .map(|data| {
            let (root, elements, outcome): (
                Option<Digest128>,
                Vec<(Digest128, Digest128)>,
                Digest128,
            ) = match arm {
                Arm::RecoveryConsult | Arm::RecoveryIgnore | Arm::FullReadSet => {
                    let (calls, outcome) = match arm {
                        Arm::RecoveryConsult => (&data.consult_calls, data.consult_outcome),
                        Arm::RecoveryIgnore => (&data.ignore_calls, data.ignore_outcome),
                        Arm::FullReadSet | Arm::RecoveryPair | Arm::WalkUnits | Arm::AllUnits => {
                            (&data.full_calls, data.verdict_outcome)
                        }
                    };
                    for call in calls {
                        descriptions.insert(call.position, call.description);
                    }
                    (
                        None,
                        calls
                            .iter()
                            .map(|call| (call.position, call.digest))
                            .collect(),
                        outcome,
                    )
                }
                Arm::WalkUnits | Arm::AllUnits | Arm::RecoveryPair => {
                    let set = match arm {
                        Arm::WalkUnits => &data.walk_elements,
                        Arm::AllUnits
                        | Arm::RecoveryPair
                        | Arm::RecoveryConsult
                        | Arm::RecoveryIgnore
                        | Arm::FullReadSet => &data.unit_elements,
                    };
                    for (position, _element, description) in set {
                        descriptions.insert(*position, *description);
                    }
                    (
                        Some(data.root),
                        set.iter()
                            .map(|(position, element, _description)| (*position, *element))
                            .collect(),
                        data.verdict_outcome,
                    )
                }
            };
            let key = key_without(root, &elements, None);
            let mut positions: Vec<Digest128> =
                elements.iter().map(|(position, _)| *position).collect();
            positions.sort_unstable();
            positions.dedup();
            ControlArmState {
                key,
                outcome,
                root,
                elements,
                positions,
            }
        })
        .collect();
    (states, descriptions)
}

/// 去掉位置 `removed` 上的全部元素再算键；读集臂与 [`sequence_key`] 同一个算法，集合臂与 [`set_key`] 同一个。
fn key_without(
    root: Option<Digest128>,
    elements: &[(Digest128, Digest128)],
    removed: Option<Digest128>,
) -> Digest128 {
    let kept = || {
        elements
            .iter()
            .filter(move |(position, _)| Some(*position) != removed)
            .map(|(_, element)| *element)
    };
    let count = kept().count();
    match root {
        Some(root) => set_key_of(root, count, kept()),
        None => {
            let mut builder = DigestBuilder::new(DIGEST_TAG_KEY_SEQUENCE);
            builder.feed_word(u64::try_from(count).expect("个数"));
            for element in kept() {
                builder.feed_digest(element);
            }
            builder.finish()
        }
    }
}

/// 不一致数：状态按序号次序，同键里第一个是参照。
fn inconsistent_count(keys_and_outcomes: impl Iterator<Item = (Digest128, Digest128)>) -> u64 {
    let mut reference: HashMap<Digest128, Digest128> = HashMap::new();
    let mut inconsistent = 0;
    for (key, outcome) in keys_and_outcomes {
        let first = reference.entry(key).or_insert(outcome);
        if *first != outcome {
            inconsistent += 1;
        }
    }
    inconsistent
}

/// PC2c / PC4c 的结果：位置总数、使不一致数 ≥ 1 的位置个数与前十个（按位置原样排序）。
struct DiscriminatingPositions {
    positions: usize,
    discriminating: Vec<PositionDescription>,
    baseline_inconsistent: u64,
}

fn discriminating_positions(
    control: &[ControlStateData],
    arm: Arm,
    threads: usize,
) -> DiscriminatingPositions {
    let (states, descriptions) = control_arm_states(control, arm);
    let baseline_inconsistent =
        inconsistent_count(states.iter().map(|state| (state.key, state.outcome)));
    let all_positions: Vec<Digest128> = descriptions.keys().copied().collect();
    let chunk = all_positions.len().div_ceil(threads.max(1)).max(1);
    let found = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for positions in all_positions.chunks(chunk) {
            let states = &states;
            let found = &found;
            scope.spawn(move || {
                let mut local = Vec::new();
                for removed in positions {
                    let inconsistent = inconsistent_count(states.iter().map(|state| {
                        let key = if state.positions.binary_search(removed).is_ok() {
                            key_without(state.root, &state.elements, Some(*removed))
                        } else {
                            state.key
                        };
                        (key, state.outcome)
                    }));
                    if inconsistent >= 1 {
                        local.push(*removed);
                    }
                }
                found.lock().expect("收集判别位置的锁").extend(local);
            });
        }
    });
    let mut discriminating: Vec<PositionDescription> = found
        .into_inner()
        .expect("收集判别位置的锁")
        .iter()
        .map(|position| descriptions[position])
        .collect();
    discriminating.sort_unstable();
    DiscriminatingPositions {
        positions: all_positions.len(),
        discriminating,
        baseline_inconsistent,
    }
}

// ───────────────────────────── 锚点 A2–A7 的值（第七节 7.2，登记第十三节 anchors.py / anchors2.py 的原样输出） ─────────────────────────────

const ANCHOR_FIRST_STREAM_SEGMENT_STATES: [u64; 10] = [8, 3, 1, 8, 3, 1, 150_994_943, 3, 1, 8];
const ANCHOR_FIRST_STREAM_STATES: u64 = 150_994_980;
/// txg1、txg2、txg3、最后一条根之后（再加全部持久那一个）。
const ANCHOR_FIRST_STREAM_BY_PUBLISH: [u64; 4] = [12, 12, 150_994_947, 8];
const ANCHOR_SECOND_STREAM_SEGMENT_STATES: [u64; 14] = [
    150_994_943,
    3,
    1,
    8,
    8,
    262_143,
    3,
    1,
    589_823,
    3,
    1,
    589_823,
    3,
    1,
];
const ANCHOR_SECOND_STREAM_STATES: u64 = 152_436_764;
/// txg4、txg5、txg6、txg7。
const ANCHOR_SECOND_STREAM_BY_PUBLISH: [u64; 4] = [150_994_947, 262_163, 589_827, 589_827];
const ANCHOR_FIRST_STREAM_SMALL: u64 = 37;
const ANCHOR_SECOND_STREAM_SMALL: u64 = 1_441_821;
const ANCHOR_STRIDE_PER_SEGMENT: u64 = 65_338;
const ANCHOR_FIRST_STREAM_QUICK: u64 = 54;
const ANCHOR_SECOND_STREAM_QUICK: u64 = 84;
/// A5：取三态的写数（第一条流 8 次系统配置槽写，第二条流 46 次）。
const ANCHOR_FIRST_STREAM_IN_PLACE_OVERWRITES: usize = 8;
const ANCHOR_SECOND_STREAM_IN_PLACE_OVERWRITES: usize = 46;
/// A6：容量的字节数。
const ANCHOR_LARGEST_CARD_BYTES: u128 = 34_190_917_632;
const ANCHOR_SIXTEEN_GIGABYTE_CARD_BYTES: u128 = 17_094_934_528;
const ANCHOR_FIVE_CARDS_BYTES: u128 = 102_595_821_568;

/// 按发布分段：每一段归它自己或它之后最近的那次根槽写所在的发布；最后一条根之后的段单列一格。
fn states_by_publish(stream: &PreparedStream, counts_by_segment: &[u64]) -> Vec<u64> {
    let mut groups = Vec::new();
    let mut running = 0u64;
    for (segment, count) in stream.segments.iter().zip(counts_by_segment) {
        running += count;
        if segment
            .iter()
            .any(|write_index| stream.writes[*write_index].kind == StepKind::RootRecordFua)
        {
            groups.push(running);
            running = 0;
        }
    }
    groups.push(running);
    groups
}

fn bool_text(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

/// 一条流的锚点 A1、A2 / A3、A5 与 A4 的域大小；交回全部都对上没有。
fn report_stream_anchors(
    output: &mut ResultLines,
    stream: &PreparedStream,
    table: &TornWriteTable,
) -> bool {
    let checks = segment_count_checks(stream, table);
    let mut all_three_agree = true;
    for check in &checks {
        let agree = check.counted_by_the_plan == check.counted_by_the_crates
            && check.counted_by_the_crates == check.counted_by_the_formula;
        all_three_agree &= agree;
        output.line(format!(
            "name=anchor_segment stream={} segment={} in_domain_counted_one_by_one={} writes={} in_place_overwrites={} counted_by_the_plan={} counted_by_the_crates={} counted_by_the_formula={} three_agree={}",
            stream.name.label(),
            check.segment_index,
            bool_text(is_sampled_segment(stream.name, check.segment_index)),
            check.writes,
            check.in_place_overwrites,
            check.counted_by_the_plan,
            check.counted_by_the_crates,
            check.counted_by_the_formula,
            bool_text(agree),
        ));
    }
    let full = LocalPlan::new(
        stream,
        table,
        domain_expansions(stream, LocalExpansion::EveryCombination),
    );
    let quick = LocalPlan::new(
        stream,
        table,
        domain_expansions(
            stream,
            LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll,
        ),
    );
    let sampled_counts: Vec<u64> = full
        .count_by_segment
        .iter()
        .enumerate()
        .filter(|(segment_index, _)| is_sampled_segment(stream.name, *segment_index))
        .map(|(_, count)| *count)
        .collect();
    let domain_states =
        full.expanded_state_count + u64::from(includes_every_write_persisted(stream.name));
    let by_publish = states_by_publish(stream, &full.count_by_segment);
    let in_place = table
        .is_in_place_overwrite
        .iter()
        .filter(|is_in_place| **is_in_place)
        .count();
    let small = u64::try_from(small_domain_ordinals(stream, &full).len()).expect("个数");
    let stride = u64::try_from(stride_domain_ordinals(stream, &full).len()).expect("个数");
    let quick_states = u64::try_from(every_ordinal(stream, &quick).len()).expect("个数");
    let (segments_ok, states_ok, publish_ok, small_ok, quick_ok, in_place_ok) = match stream.name {
        StreamName::First => (
            sampled_counts == ANCHOR_FIRST_STREAM_SEGMENT_STATES,
            domain_states == ANCHOR_FIRST_STREAM_STATES,
            by_publish == ANCHOR_FIRST_STREAM_BY_PUBLISH,
            small == ANCHOR_FIRST_STREAM_SMALL,
            quick_states == ANCHOR_FIRST_STREAM_QUICK,
            in_place == ANCHOR_FIRST_STREAM_IN_PLACE_OVERWRITES,
        ),
        StreamName::Second => (
            sampled_counts == ANCHOR_SECOND_STREAM_SEGMENT_STATES,
            domain_states == ANCHOR_SECOND_STREAM_STATES,
            by_publish
                .iter()
                .copied()
                .filter(|count| *count > 0)
                .collect::<Vec<u64>>()
                == ANCHOR_SECOND_STREAM_BY_PUBLISH,
            small == ANCHOR_SECOND_STREAM_SMALL,
            quick_states == ANCHOR_SECOND_STREAM_QUICK,
            in_place == ANCHOR_SECOND_STREAM_IN_PLACE_OVERWRITES,
        ),
    };
    let stride_ok = stride == ANCHOR_STRIDE_PER_SEGMENT;
    let mask_width_formula_holds =
        table.mask_width() == stream.writes.len() + in_place + table.replay_count;
    output.line(format!(
        "name=anchor_stream stream={} sampled_segment_states={sampled_counts:?} domain_states={domain_states} by_publish={by_publish:?} d1_small={small} d1_stride={stride} d1_quick={quick_states} recorded_writes={} in_place_overwrites={in_place} replays={} w={} segments_match={} domain_states_match={} by_publish_match={} d1_small_match={} d1_stride_match={} d1_quick_match={} in_place_overwrites_match={} w_formula_holds={}",
        stream.name.label(),
        stream.writes.len(),
        table.replay_count,
        table.mask_width(),
        bool_text(segments_ok),
        bool_text(states_ok),
        bool_text(publish_ok),
        bool_text(small_ok),
        bool_text(stride_ok),
        bool_text(quick_ok),
        bool_text(in_place_ok),
        bool_text(mask_width_formula_holds),
    ));
    all_three_agree
        && segments_ok
        && states_ok
        && publish_ok
        && small_ok
        && stride_ok
        && quick_ok
        && in_place_ok
        && mask_width_formula_holds
}

/// A6、A7：容量与最大一段的算术。
fn capacity_anchors_hold() -> bool {
    LARGEST_CARD_MEBIBYTES * MEBIBYTE == ANCHOR_LARGEST_CARD_BYTES
        && SMALLEST_SIXTEEN_GIGABYTE_CARD_MEBIBYTES * MEBIBYTE == ANCHOR_SIXTEEN_GIGABYTE_CARD_BYTES
        && FIVE_CARDS_MEBIBYTES * MEBIBYTE == ANCHOR_FIVE_CARDS_BYTES
        && 3u128.pow(2) * 2u128.pow(28) == SECOND_STREAM_LARGEST_SEGMENT_STATES + 1
}

// ───────────────────────────── 模式 stop-clauses：S1–S6 与 A1–A7（跑产物之前） ─────────────────────────────

/// S3 + S4 在一个域上：crates 的枚举与装置自己照跑的计数逐项比。交回（S3 过没过，S4 过没过）。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：输出、格名、输入、计划、序号、crates 的展开、线程数"
)]
fn report_crates_comparison(
    output: &mut ResultLines,
    label: &str,
    context: &EvaluationContext<'_>,
    plan: &LocalPlan<'_>,
    ordinals: &[u64],
    expansion_of: &dyn Fn(usize) -> Layer0SegmentExpansion,
    threads: usize,
) -> (bool, bool) {
    let comparison =
        compare_with_the_crates(context.stream, context.table, plan, ordinals, expansion_of);
    let device = run_cell(&CellRun {
        label: format!("{label}-baseline"),
        context,
        plan,
        ordinals,
        control_ordinals: &BTreeSet::new(),
        busy_zone: None,
        threads,
        depth: EvaluationDepth::BaselineOnly,
    });
    let device_tally = tally_with_every_write_persisted(context, &device.tally);
    let device_text = tally_text(&device_tally);
    let crates_text = tally_text(&comparison.crates_tally);
    let expected_observed = u64::try_from(ordinals.len()).expect("个数")
        + u64::from(!includes_every_write_persisted(context.stream.name));
    let enumeration_clause_holds = !comparison.writes_table_differs
        && comparison.persisted_mismatches == 0
        && comparison.compared_states == u64::try_from(ordinals.len()).expect("个数")
        && comparison.observed_states == expected_observed;
    let tally_clause_holds = device_text == crates_text;
    output.line(format!(
        "name=stop_clause_crates cell={label} domain_states={} crates_observed_states={} compared_states={} writes_table_differs={} persisted_mismatches={} clause_S3_holds={} clause_S4_holds={}",
        ordinals.len(),
        comparison.observed_states,
        comparison.compared_states,
        bool_text(comparison.writes_table_differs),
        comparison.persisted_mismatches,
        bool_text(enumeration_clause_holds),
        bool_text(tally_clause_holds),
    ));
    output.line(format!(
        "name=stop_clause_tally cell={label} side=device {device_text}"
    ));
    output.line(format!(
        "name=stop_clause_tally cell={label} side=crates {crates_text}"
    ));
    (enumeration_clause_holds, tally_clause_holds)
}

/// 对照样本上跑两遍（5.1 的四件事），第二遍与第一遍逐状态比键与结局（PC2a、PC4a）、两遍的照跑结局（PC2d、PC4d）；
/// 第一遍自己核 S5（记录跑与照跑逐项相同）与 S6（Q4e）。交回（S5 过没过，S6 过没过）。
fn report_control_double_run(
    output: &mut ResultLines,
    label: &str,
    context: &EvaluationContext<'_>,
    plan: &LocalPlan<'_>,
    control: &BTreeSet<u64>,
    threads: usize,
) -> (bool, bool) {
    let ordinals: Vec<u64> = control.iter().copied().collect();
    let run_once = |pass: &str| {
        run_cell(&CellRun {
            label: format!("{label}-control-{pass}"),
            context,
            plan,
            ordinals: &ordinals,
            control_ordinals: control,
            busy_zone: None,
            threads,
            depth: EvaluationDepth::FourSteps,
        })
    };
    let first = run_once("first");
    let second = run_once("second");
    let keys_of = |data: &ControlStateData| {
        [
            sequence_key(&data.consult_calls),
            sequence_key(&data.ignore_calls),
            set_key(data.root, &data.walk_elements),
            set_key(data.root, &data.unit_elements),
            sequence_key(&data.full_calls),
        ]
    };
    let mut recovery_key_changed = 0u64;
    let mut checker_key_changed = 0u64;
    let mut recovery_outcome_changed = 0u64;
    let mut verdict_changed = 0u64;
    for (left, right) in first.control.iter().zip(&second.control) {
        assert_eq!(left.ordinal, right.ordinal, "两遍对照样本按同一个次序");
        let (left_keys, right_keys) = (keys_of(left), keys_of(right));
        recovery_key_changed += u64::from(left_keys[..2] != right_keys[..2]);
        checker_key_changed += u64::from(left_keys[2..] != right_keys[2..]);
        recovery_outcome_changed += u64::from(
            left.consult_outcome != right.consult_outcome
                || left.ignore_outcome != right.ignore_outcome,
        );
        verdict_changed += u64::from(left.verdict_outcome != right.verdict_outcome);
    }
    let compared = u64::try_from(first.control.len().min(second.control.len())).expect("个数");
    let determinism_clause_holds = first.consult_report_differs == 0
        && first.ignore_report_differs == 0
        && first.verdicts_differ == 0
        && recovery_outcome_changed == 0
        && verdict_changed == 0
        && compared == u64::try_from(ordinals.len()).expect("个数");
    let walk_subset_clause_holds = first.walk_reads_outside_normal_reads == 0;
    output.line(format!(
        "name=stop_clause_determinism cell={label} control_states={} compared_states={compared} recorded_consult_report_differs={} recorded_ignore_report_differs={} recorded_verdicts_differ={} second_run_recovery_outcome_changed={recovery_outcome_changed} second_run_verdict_changed={verdict_changed} walk_unit_reads_outside_normal_unit_reads={} clause_S5_holds={} clause_S6_holds={}",
        ordinals.len(),
        first.consult_report_differs,
        first.ignore_report_differs,
        first.verdicts_differ,
        first.walk_reads_outside_normal_reads,
        bool_text(determinism_clause_holds),
        bool_text(walk_subset_clause_holds),
    ));
    output.line(format!(
        "name=positive_control_same_state_twice cell={label} states={compared} pc2a_recovery_keys_changed={recovery_key_changed} pc4a_checker_keys_changed={checker_key_changed} pc2d_recovery_reports_changed={recovery_outcome_changed} pc4d_verdicts_changed={verdict_changed} pc2a_holds={} pc4a_holds={} pc2d_holds={} pc4d_holds={}",
        bool_text(recovery_key_changed == 0 && compared > 0),
        bool_text(checker_key_changed == 0 && compared > 0),
        bool_text(recovery_outcome_changed == 0 && compared > 0),
        bool_text(verdict_changed == 0 && compared > 0),
    ));
    (determinism_clause_holds, walk_subset_clause_holds)
}

fn run_stop_clauses(output: &mut ResultLines) -> i32 {
    let threads = threads_from_environment();
    output.line(format!(
        "name=config experiment=E161 mode=stop_clauses threads={threads} registration=research/prompts/e161-preregistration.md"
    ));
    let constants = local_constants_against_the_crates();
    let constants_clause_holds = constants
        .iter()
        .all(|(_name, local, crates)| local == crates);
    let constant_fields: Vec<String> = constants
        .iter()
        .map(|(name, local, crates)| format!("{name}={local}/{crates}"))
        .collect();
    output.line(format!(
        "name=stop_clause_constants {} clause_S1_holds={}",
        constant_fields.join(" "),
        bool_text(constants_clause_holds)
    ));
    let first = prepare_first_stream();
    let second = prepare_second_stream();
    let (bases_identical, prefix_identical) = shared_prefix_is_identical(&first, &second);
    let mut streams_clause_holds = bases_identical && prefix_identical;
    for stream in [&first, &second] {
        let check = check_stream_shape(stream);
        streams_clause_holds &=
            check.segment_lengths_match && check.writes_match && check.roots_match;
        output.line(format!(
            "name=stop_clause_stream stream={} writes={} segments={} roots={} segment_lengths_match={} writes_match={} roots_match={}",
            stream.name.label(),
            stream.writes.len(),
            stream.segments.len(),
            stream.root_write_count,
            bool_text(check.segment_lengths_match),
            bool_text(check.writes_match),
            bool_text(check.roots_match),
        ));
    }
    output.line(format!(
        "name=stop_clause_shared_prefix bases_identical={} first_nine_segments_identical={} clause_S2_holds={}",
        bool_text(bases_identical),
        bool_text(prefix_identical),
        bool_text(streams_clause_holds)
    ));
    let mut anchors_hold = capacity_anchors_hold();
    output.line(format!(
        "name=anchor_capacity a6_a7_hold={}",
        bool_text(anchors_hold)
    ));
    let mut enumeration_clause_holds = true;
    let mut tally_clause_holds = true;
    let mut determinism_clause_holds = true;
    let mut walk_subset_clause_holds = true;
    for stream in [&first, &second] {
        let table = TornWriteTable::of(stream);
        anchors_hold &= report_stream_anchors(output, stream, &table);
        let context = EvaluationContext::new(stream, &table);
        let full = LocalPlan::new(
            stream,
            &table,
            domain_expansions(stream, LocalExpansion::EveryCombination),
        );
        let small = small_domain_ordinals(stream, &full);
        let small_segments: BTreeSet<usize> = small
            .iter()
            .map(|ordinal| full.segment_of(*ordinal))
            .collect();
        let (small_enumeration_holds, small_tally_holds) = report_crates_comparison(
            output,
            &format!("{}_small", stream.name.label()),
            &context,
            &full,
            &small,
            &|segment_index| {
                if small_segments.contains(&segment_index) {
                    Layer0SegmentExpansion::EveryProperSubset
                } else {
                    Layer0SegmentExpansion::NotExpanded
                }
            },
            threads,
        );
        enumeration_clause_holds &= small_enumeration_holds;
        tally_clause_holds &= small_tally_holds;
        let quick = LocalPlan::new(
            stream,
            &table,
            domain_expansions(
                stream,
                LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll,
            ),
        );
        let quick_ordinals = every_ordinal(stream, &quick);
        let (quick_enumeration_holds, quick_tally_holds) = report_crates_comparison(
            output,
            &format!("{}_quick", stream.name.label()),
            &context,
            &quick,
            &quick_ordinals,
            &|segment_index| quick.expansion_by_segment[segment_index].crates_expansion(),
            threads,
        );
        enumeration_clause_holds &= quick_enumeration_holds;
        tally_clause_holds &= quick_tally_holds;
        let mut domain: Vec<u64> = small
            .iter()
            .chain(&stride_domain_ordinals(stream, &full))
            .copied()
            .collect();
        domain.sort_unstable();
        let control = control_ordinals(stream, &full, &domain);
        let (control_determinism_holds, control_walk_subset_holds) = report_control_double_run(
            output,
            &format!("{}_d1", stream.name.label()),
            &context,
            &full,
            &control,
            threads,
        );
        determinism_clause_holds &= control_determinism_holds;
        walk_subset_clause_holds &= control_walk_subset_holds;
    }
    output.line(format!(
        "name=verdict scope=stop_clauses clause_S1_holds={} clause_S2_holds={} clause_S3_holds={} clause_S4_holds={} clause_S5_holds={} clause_S6_holds={} anchors_a1_to_a7_hold={}",
        bool_text(constants_clause_holds),
        bool_text(streams_clause_holds),
        bool_text(enumeration_clause_holds),
        bool_text(tally_clause_holds),
        bool_text(determinism_clause_holds),
        bool_text(walk_subset_clause_holds),
        bool_text(anchors_hold),
    ));
    if constants_clause_holds
        && streams_clause_holds
        && enumeration_clause_holds
        && tally_clause_holds
        && determinism_clause_holds
        && anchors_hold
    {
        0
    } else {
        3
    }
}

/// 开发用：只在第一条流与第二条流的甲二域上跑 S3、S4 与对照样本（第一条流）的两遍，几分钟之内看出装置有没有写错；不出产物。
fn run_first_stream_stop_clause_probe(output: &mut ResultLines) -> i32 {
    let threads = threads_from_environment();
    let first = prepare_first_stream();
    let second = prepare_second_stream();
    for stream in [&first, &second] {
        let table = TornWriteTable::of(stream);
        let _anchors = report_stream_anchors(output, stream, &table);
        let context = EvaluationContext::new(stream, &table);
        let quick = LocalPlan::new(
            stream,
            &table,
            domain_expansions(
                stream,
                LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll,
            ),
        );
        let quick_ordinals = every_ordinal(stream, &quick);
        let _quick = report_crates_comparison(
            output,
            &format!("{}_quick", stream.name.label()),
            &context,
            &quick,
            &quick_ordinals,
            &|segment_index| quick.expansion_by_segment[segment_index].crates_expansion(),
            threads,
        );
    }
    let table = TornWriteTable::of(&first);
    let context = EvaluationContext::new(&first, &table);
    let full = LocalPlan::new(
        &first,
        &table,
        domain_expansions(&first, LocalExpansion::EveryCombination),
    );
    let small = small_domain_ordinals(&first, &full);
    let small_segments: BTreeSet<usize> = small
        .iter()
        .map(|ordinal| full.segment_of(*ordinal))
        .collect();
    let _small = report_crates_comparison(
        output,
        "first_small",
        &context,
        &full,
        &small,
        &|segment_index| {
            if small_segments.contains(&segment_index) {
                Layer0SegmentExpansion::EveryProperSubset
            } else {
                Layer0SegmentExpansion::NotExpanded
            }
        },
        threads,
    );
    let control: BTreeSet<u64> = small.iter().copied().collect();
    let _control =
        report_control_double_run(output, "first_small", &context, &full, &control, threads);
    0
}

// ───────────────────────────── G3 的加权估计（5.5 第一段） ─────────────────────────────

/// 单元级检查占每状态总时长的上下界估计（Q3i）：各段取样状态的平均时长 × 该段状态数，再加总。
#[derive(Clone, Copy, Debug)]
struct UnitShareEstimate {
    lower: f64,
    upper: f64,
}

/// G3 的判定（第六节 Q3i）：s_hi < 0.5 翻；s_lo ≥ 0.5 不翻；其余不够判。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShareVerdict {
    FlipsToWalkAndJudgeOnTheDeviceToo,
    DoesNotFlip,
    NotEnoughToJudge,
}

impl ShareVerdict {
    fn of(estimate: UnitShareEstimate, threshold: f64) -> Self {
        if estimate.upper < threshold {
            Self::FlipsToWalkAndJudgeOnTheDeviceToo
        } else if estimate.lower >= threshold {
            Self::DoesNotFlip
        } else {
            Self::NotEnoughToJudge
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::FlipsToWalkAndJudgeOnTheDeviceToo => "flips_walk_and_judge_on_the_device_too",
            Self::DoesNotFlip => "does_not_flip",
            Self::NotEnoughToJudge => "not_enough_to_judge",
        }
    }
}

fn threshold_half() -> f64 {
    f64::from(u32::try_from(RATIO_THRESHOLD_NUMERATOR).expect("门槛分子"))
        / f64::from(u32::try_from(RATIO_THRESHOLD_DENOMINATOR).expect("门槛分母"))
}

#[allow(
    clippy::cast_precision_loss,
    reason = "计时的纳秒和装得进 f64 的 53 位尾数之内（D1 上最多几千秒 ≈ 10¹³ 纳秒），这里只算占比"
)]
fn nanoseconds_as_float(nanoseconds: u128) -> f64 {
    nanoseconds as f64
}

#[allow(
    clippy::cast_precision_loss,
    reason = "段的状态数至多 3² · 2²⁴，装得进 f64 的尾数"
)]
fn count_as_float(count: u64) -> f64 {
    count as f64
}

/// 加权估计：`timing_by_segment` 是取样状态的各段累计，`plan` 给各段在全域里的状态数（全部持久那一个算一段、1 个）。
fn weighted_unit_share(
    timing_by_segment: &BTreeMap<usize, TimingSums>,
    plan: &LocalPlan<'_>,
) -> UnitShareEstimate {
    let mut state = 0f64;
    let mut positions = 0f64;
    let mut events = 0f64;
    for (segment, sums) in timing_by_segment {
        if sums.states == 0 {
            continue;
        }
        let states_in_the_full_domain = plan.count_by_segment.get(*segment).copied().unwrap_or(1);
        let weight = count_as_float(states_in_the_full_domain) / count_as_float(sums.states);
        state += nanoseconds_as_float(sums.state_nanoseconds) * weight;
        positions += nanoseconds_as_float(sums.zone_nanoseconds[5]) * weight;
        events += nanoseconds_as_float(sums.unit_events_replay_nanoseconds) * weight;
    }
    UnitShareEstimate {
        lower: positions / state,
        upper: events / state,
    }
}

/// 未加权的上下界（D1-quick 与线程数 1 那一格：完整枚举或同一段里的取样，不用加权）。
fn unweighted_unit_share(summary: &CellSummary) -> UnitShareEstimate {
    UnitShareEstimate {
        lower: nanoseconds_as_float(summary.zone_nanoseconds[5])
            / nanoseconds_as_float(summary.state_nanoseconds),
        upper: nanoseconds_as_float(summary.unit_events_replay_nanoseconds)
            / nanoseconds_as_float(summary.state_nanoseconds),
    }
}

/// 第一段 G3 够不够判：两个数在 0.5 同一侧且离 0.5 至少 0.1（5.5）。
fn share_is_enough_to_judge(estimate: UnitShareEstimate) -> bool {
    let threshold = threshold_half();
    (estimate.upper < threshold - SHARE_MARGIN_FOR_ENOUGH)
        || (estimate.lower >= threshold + SHARE_MARGIN_FOR_ENOUGH)
}

// ───────────────────────────── 模式 segment-one：一条流上的 D1 与 D1-quick ─────────────────────────────

/// 一条流跑完第一段留下的东西。
struct StreamOutcome {
    name: StreamName,
    write_table_length: usize,
    small: CellSummary,
    stride: CellSummary,
    union: CellSummary,
    quick: CellSummary,
    weighted: UnitShareEstimate,
    pc1: Option<(bool, bool, u64)>,
}

fn stream_label(stream: StreamName, domain: &str) -> String {
    format!("{}_{domain}", stream.label())
}

/// 报出 PC2c / PC4c 的一行。
fn report_discriminating_positions(
    output: &mut ResultLines,
    label: &str,
    control: &[ControlStateData],
    threads: usize,
) -> Vec<(Arm, bool)> {
    let mut found = Vec::new();
    for arm in [
        Arm::RecoveryConsult,
        Arm::RecoveryIgnore,
        Arm::WalkUnits,
        Arm::AllUnits,
        Arm::FullReadSet,
    ] {
        let result = discriminating_positions(control, arm, threads);
        let first_ten: Vec<String> = result
            .discriminating
            .iter()
            .take(10)
            .map(PositionDescription::text)
            .collect();
        let has_one = !result.discriminating.is_empty();
        output.line(format!(
            "name=positive_control_remove_one_position cell={label} arm={} control_states={} positions={} discriminating_positions={} first_ten=[{}] inconsistent_without_removal={} found_at_least_one={}",
            arm.label(),
            control.len(),
            result.positions,
            result.discriminating.len(),
            first_ten.join(","),
            result.baseline_inconsistent,
            bool_text(has_one),
        ));
        found.push((arm, has_one));
    }
    found
}

#[allow(
    clippy::too_many_lines,
    reason = "一条流上第一段的全部格按登记 5.5 的次序排下来：D1-small、D1-stride、并格、加权 G3、G5、D1-quick、PC1、PC2c / PC4c"
)]
fn process_stream(
    output: &mut ResultLines,
    stream: &PreparedStream,
    threads: usize,
) -> (StreamOutcome, Vec<(Arm, bool)>) {
    let table = TornWriteTable::of(stream);
    let context = EvaluationContext::new(stream, &table);
    let full = LocalPlan::new(
        stream,
        &table,
        domain_expansions(stream, LocalExpansion::EveryCombination),
    );
    let small = small_domain_ordinals(stream, &full);
    let stride = stride_domain_ordinals(stream, &full);
    let mut domain: Vec<u64> = small.iter().chain(&stride).copied().collect();
    domain.sort_unstable();
    let control = control_ordinals(stream, &full, &domain);
    let strided_segments: BTreeSet<usize> = stride
        .iter()
        .map(|ordinal| full.segment_of(*ordinal))
        .collect();
    let run =
        |label: &str, ordinals: &[u64], control_ordinals: &BTreeSet<u64>, plan: &LocalPlan<'_>| {
            run_cell(&CellRun {
                label: label.to_string(),
                context: &context,
                plan,
                ordinals,
                control_ordinals,
                busy_zone: None,
                threads,
                depth: EvaluationDepth::FourSteps,
            })
        };
    let mut small_totals = run(&stream_label(stream.name, "small"), &small, &control, &full);
    let mut stride_totals = run(
        &stream_label(stream.name, "stride"),
        &stride,
        &control,
        &full,
    );
    let mut union_stores: Vec<ArmRecordStore> = ARMS
        .iter()
        .map(|_arm| ArmRecordStore::new(ARM_RECORDS_IN_MEMORY_BEFORE_SPILL))
        .collect();
    let small_stores = std::mem::take(&mut small_totals.arm_stores);
    let small_summary = report_cell(
        output,
        &stream_label(stream.name, "small"),
        &small_totals,
        small_stores,
        &TrajectoryIntervals::of(&full, &small, &strided_segments),
        Some(&mut union_stores),
    );
    let stride_stores = std::mem::take(&mut stride_totals.arm_stores);
    let stride_summary = report_cell(
        output,
        &stream_label(stream.name, "stride"),
        &stride_totals,
        stride_stores,
        &TrajectoryIntervals::of(&full, &stride, &strided_segments),
        Some(&mut union_stores),
    );
    let mut union_totals = small_totals;
    union_totals.absorb(stride_totals);
    let union_summary = report_cell(
        output,
        &stream_label(stream.name, "d1"),
        &union_totals,
        union_stores,
        &TrajectoryIntervals::of(&full, &domain, &strided_segments),
        None,
    );
    let weighted = weighted_unit_share(&union_totals.timing_by_segment, &full);
    output.line(format!(
        "name=g3_weighted stream={} unit_share_lower_ratio={:.6} unit_share_upper_ratio={:.6}",
        stream.name.label(),
        weighted.lower,
        weighted.upper,
    ));
    for arm in [Arm::WalkUnits, Arm::AllUnits, Arm::FullReadSet] {
        let _verdicts = report_device_memory(
            output,
            &union_summary,
            table.mask_width(),
            SECOND_STREAM_WRITES + ANCHOR_SECOND_STREAM_IN_PLACE_OVERWRITES,
            arm,
        );
    }
    let _device_memory_control_holds = report_device_memory_control(
        output,
        &stream_label(stream.name, "d1"),
        &mut union_totals.tables,
        &union_summary,
        table.mask_width(),
    );
    let quick_plan = LocalPlan::new(
        stream,
        &table,
        domain_expansions(
            stream,
            LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll,
        ),
    );
    let quick_ordinals = every_ordinal(stream, &quick_plan);
    let mut quick_totals = run(
        &stream_label(stream.name, "quick"),
        &quick_ordinals,
        &BTreeSet::new(),
        &quick_plan,
    );
    let quick_stores = std::mem::take(&mut quick_totals.arm_stores);
    let quick_summary = report_cell(
        output,
        &stream_label(stream.name, "quick"),
        &quick_totals,
        quick_stores,
        &TrajectoryIntervals::of(&quick_plan, &quick_ordinals, &BTreeSet::new()),
        None,
    );
    for arm in [Arm::WalkUnits, Arm::AllUnits, Arm::FullReadSet] {
        let _verdicts = report_device_memory(
            output,
            &quick_summary,
            table.mask_width(),
            SECOND_STREAM_WRITES + ANCHOR_SECOND_STREAM_IN_PLACE_OVERWRITES,
            arm,
        );
    }
    let discriminating = report_discriminating_positions(
        output,
        &stream_label(stream.name, "d1_control"),
        &union_totals.control,
        threads,
    );
    let pc1 = includes_every_write_persisted(stream.name).then(|| {
        refeed_every_write_persisted(
            &context,
            &full,
            &union_totals.tables,
            union_totals.whole_unit_positions,
        )
    });
    if let Some((distinct_unchanged, checks_grew_exactly, positions)) = pc1 {
        output.line(format!(
            "name=positive_control_refeed cell={} refed_state_unit_positions={positions} distinct_unit_contents_unchanged={} per_state_checks_grew_by_exactly_its_positions={} pc1_holds={}",
            stream_label(stream.name, "d1"),
            bool_text(distinct_unchanged),
            bool_text(checks_grew_exactly),
            bool_text(distinct_unchanged && checks_grew_exactly),
        ));
    }
    let outcome = StreamOutcome {
        name: stream.name,
        write_table_length: table.mask_width(),
        small: small_summary,
        stride: stride_summary,
        union: union_summary,
        quick: quick_summary,
        weighted,
        pc1,
    };
    (outcome, discriminating)
}

/// PC1：第一条流全部持久那个状态当一个额外状态再喂一遍（新的序号），数去重臂的不同内容数变没变、逐状态臂加了多少。
/// 交回（不同内容数没变，逐状态次数恰好加上它的不同单元位置数，它的不同单元位置数）。
fn refeed_every_write_persisted(
    context: &EvaluationContext<'_>,
    plan: &LocalPlan<'_>,
    union_tables: &ContentTables,
    per_state_checks_before: u64,
) -> (bool, bool, u64) {
    let refed_ordinal = u64::MAX - 1;
    let tables = RefCell::new(ContentTables::default());
    let mut tally = Layer0Tally::default();
    let result = evaluate_state(
        context,
        refed_ordinal,
        context.stream.segments.len(),
        plan.persisted_of(plan.expanded_state_count),
        None,
        false,
        &tables,
        &mut tally,
    );
    let checker_contents = |table: &ContentTable| -> BTreeSet<Digest128> {
        table
            .entries
            .iter()
            .filter(|(_, entry)| entry.first_ordinal_checker_normal_pass.is_some())
            .map(|(digest, _)| *digest)
            .collect()
    };
    let before = checker_contents(&union_tables.by_category[ReadCategory::WholeUnit.slot()]);
    let refed = checker_contents(&tables.into_inner().by_category[ReadCategory::WholeUnit.slot()]);
    let after: BTreeSet<Digest128> = before.union(&refed).copied().collect();
    let positions = result.whole_unit_positions;
    let mut extra = CellTotals::new();
    extra.absorb_state(result);
    let per_state_checks_after = per_state_checks_before + extra.whole_unit_positions;
    (
        after.len() == before.len(),
        per_state_checks_after == per_state_checks_before + positions && positions > 0,
        positions,
    )
}

// ───────────────────────────── PC5、PC3 ─────────────────────────────

/// P1 的一类：表里被恢复或 checker 正常那一遍读到过的内容的字节数之和。
fn content_bytes(table: &ContentTable) -> u128 {
    table
        .entries
        .values()
        .filter(|entry| entry.first_ordinal_any_pass.is_some())
        .map(|entry| u128::try_from(entry.bytes.len()).expect("长度"))
        .sum()
}

/// 合成的内容：32768 字节全 0xA5（PC5）。
const SYNTHETIC_CONTENT_BYTE: u8 = 0xA5;
/// 合成的读集：7 次调用（PC5）。
const SYNTHETIC_READ_SET_CALLS: u64 = 7;

/// PC5：往 P1 加一份从没读到过的 32768 字节、往 P3 加一条 7 次调用的合成读集、多记一个合成状态，四部分各自恰好涨多少。
fn report_device_memory_control(
    output: &mut ResultLines,
    label: &str,
    tables: &mut ContentTables,
    summary: &CellSummary,
    write_table_length: usize,
) -> bool {
    let whole_unit = ReadCategory::WholeUnit.slot();
    let contents_before: u128 = tables.by_category.iter().map(content_bytes).sum();
    let synthetic =
        vec![SYNTHETIC_CONTENT_BYTE; usize::try_from(LOCAL_DATA_UNIT_BYTES).expect("32768")];
    tables.by_category[whole_unit].insert(digest_of_bytes(&synthetic), &synthetic, u64::MAX, false);
    let contents_after: u128 = tables.by_category.iter().map(content_bytes).sum();
    let arm = Arm::FullReadSet;
    let before = device_memory_parts(summary, write_table_length, arm);
    let mut with_read_set = summary.clone();
    with_read_set.arm_distinct_key_elements[Arm::RecoveryConsult.slot()] +=
        SYNTHETIC_READ_SET_CALLS;
    with_read_set.arm_distinct_keys[Arm::RecoveryConsult.slot()] += 1;
    let after_read_set = device_memory_parts(&with_read_set, write_table_length, arm);
    let mut with_state = summary.clone();
    with_state.states += 1;
    let after_state = device_memory_parts(&with_state, write_table_length, arm);
    let contents_grew = contents_after - contents_before;
    let read_sets_grew = after_read_set.read_sets - before.read_sets;
    let masks_grew = after_state.masks - before.masks;
    let keys_grew = after_state.keys_per_state - before.keys_per_state;
    let holds = contents_grew == u128::from(LOCAL_DATA_UNIT_BYTES)
        && read_sets_grew == u128::from(SYNTHETIC_READ_SET_CALLS * READ_SET_BYTES_PER_CALL)
        && masks_grew == mask_bytes_per_state(write_table_length)
        && keys_grew == u128::from(KEYS_STORED_PER_STATE * KEY_BYTES);
    output.line(format!(
        "name=positive_control_device_memory cell={label} p1_grew={contents_grew} p3_grew={read_sets_grew} p2_grew={masks_grew} p4_per_state_grew={keys_grew} pc5_holds={}",
        bool_text(holds)
    ));
    holds
}

/// PC3：第二条流段 14 序号最小的 1000 个状态，先照常跑一遍，再六遍各在一个计时区里加 2 ms 忙等。交回全部六个区都过没过。
fn report_timing_control(
    output: &mut ResultLines,
    stream: &PreparedStream,
    threads: usize,
) -> bool {
    let table = TornWriteTable::of(stream);
    let context = EvaluationContext::new(stream, &table);
    let full = LocalPlan::new(
        stream,
        &table,
        domain_expansions(stream, LocalExpansion::EveryCombination),
    );
    let segment = TIMING_CONTROL_SEGMENT;
    let first = full.first_ordinal_by_segment[segment];
    let ordinals: Vec<u64> =
        (first..first + u64::try_from(TIMING_CONTROL_STATES).expect("1000")).collect();
    let zone_totals = |busy_zone: Option<TimingZone>| -> [u128; 6] {
        let totals = run_cell(&CellRun {
            label: format!("pc3-{}", busy_zone.map_or("none", TimingZone::label)),
            context: &context,
            plan: &full,
            ordinals: &ordinals,
            control_ordinals: &BTreeSet::new(),
            busy_zone,
            threads,
            depth: EvaluationDepth::FourSteps,
        });
        let mut sums = TimingSums::default();
        for segment_sums in totals.timing_by_segment.values() {
            sums.absorb(segment_sums);
        }
        sums.zone_nanoseconds
    };
    let without = zone_totals(None);
    let mut every_zone_holds = true;
    for (busy_slot, busy_zone) in TIMING_ZONES.iter().enumerate() {
        let with = zone_totals(Some(*busy_zone));
        let changes: Vec<i128> = with
            .iter()
            .zip(&without)
            .map(|(after, before)| {
                i128::try_from(*after).expect("纳秒") - i128::try_from(*before).expect("纳秒")
            })
            .collect();
        let added_enough = changes[busy_slot]
            >= i128::try_from(TIMING_CONTROL_MINIMUM_ADDED_NANOSECONDS).expect("纳秒");
        let others_small = changes.iter().enumerate().all(|(slot, change)| {
            slot == busy_slot
                || change.unsigned_abs() < TIMING_CONTROL_MAXIMUM_OTHER_CHANGE_NANOSECONDS
        });
        let holds = added_enough && others_small;
        every_zone_holds &= holds;
        let change_fields: Vec<String> = TIMING_ZONES
            .iter()
            .zip(&changes)
            .map(|(zone, change)| format!("{}_delta_elapsed_ns={change}", zone.label()))
            .collect();
        output.line(format!(
            "name=positive_control_timing busy_zone={} states={} {} busy_zone_added_at_least_1800ms={} other_zones_changed_under_1000ms={} pc3_zone_holds={}",
            busy_zone.label(),
            ordinals.len(),
            change_fields.join(" "),
            bool_text(added_enough),
            bool_text(others_small),
            bool_text(holds),
        ));
    }
    every_zone_holds
}

/// PC3 取的段（5.3：第二条流段 14）。
const TIMING_CONTROL_SEGMENT: usize = 14;

// ───────────────────────────── 几何敏感性与判别力自证（8.2） ─────────────────────────────

/// 一个比值量在两个取样点上：各自的判定（门槛 0.5）、翻不翻面，门槛挪到两点之间时判定函数分不分得开。
fn ratio_sensitivity_fields(name: &str, first: (u128, u128), second: (u128, u128)) -> String {
    let first_side = reaches_half(first.0, first.1);
    let second_side = reaches_half(second.0, second.1);
    let equal = first.0 * second.1 == second.0 * first.1;
    let separates = if equal || first.1 == 0 || second.1 == 0 {
        None
    } else {
        let threshold_numerator = first.0 * second.1 + second.0 * first.1;
        let threshold_denominator = 2 * first.1 * second.1;
        Some(
            ratio_reaches(first.0, first.1, threshold_numerator, threshold_denominator)
                != ratio_reaches(
                    second.0,
                    second.1,
                    threshold_numerator,
                    threshold_denominator,
                ),
        )
    };
    format!(
        "{name}_d1={} {name}_quick={} {name}_stable={} {name}_threshold_moved_between_separates={}",
        ratio_text(first.0, first.1),
        ratio_text(second.0, second.1),
        bool_text(first_side == second_side),
        separates.map_or("threshold_cannot_move_in", bool_text),
    )
}

fn share_sensitivity_fields(
    name: &str,
    first: UnitShareEstimate,
    second: UnitShareEstimate,
) -> String {
    let threshold = threshold_half();
    let first_verdict = ShareVerdict::of(first, threshold);
    let second_verdict = ShareVerdict::of(second, threshold);
    let separates = |pick: fn(UnitShareEstimate) -> f64| -> Option<bool> {
        let (left, right) = (pick(first), pick(second));
        if (left - right).abs() < f64::EPSILON {
            return None;
        }
        let moved = (left + right) / 2.0;
        Some(ShareVerdict::of(first, moved) != ShareVerdict::of(second, moved))
    };
    format!(
        "{name}_first_point_verdict={} {name}_second_point_verdict={} {name}_stable={} {name}_lower_threshold_moved_between_separates={} {name}_upper_threshold_moved_between_separates={}",
        first_verdict.label(),
        second_verdict.label(),
        bool_text(first_verdict == second_verdict),
        separates(|estimate| estimate.lower).map_or("threshold_cannot_move_in", bool_text),
        separates(|estimate| estimate.upper).map_or("threshold_cannot_move_in", bool_text),
    )
}

fn capacity_sensitivity_fields(
    name: &str,
    first_required: u128,
    second_required: u128,
    capacity: u128,
) -> String {
    let first_verdict = CapacityVerdict::of(first_required, capacity);
    let second_verdict = CapacityVerdict::of(second_required, capacity);
    let separates = if first_required == second_required {
        None
    } else {
        let moved = first_required.midpoint(second_required);
        Some(
            CapacityVerdict::of(first_required, moved)
                != CapacityVerdict::of(second_required, moved),
        )
    };
    format!(
        "{name}_d1={} {name}_quick={} {name}_stable={} {name}_threshold_moved_between_separates={}",
        first_verdict.label(),
        second_verdict.label(),
        bool_text(first_verdict == second_verdict),
        separates.map_or("threshold_cannot_move_in", bool_text),
    )
}

/// 8.2 展开方式那一行（D1 与 D1-quick）：R1、K_pair / N、三条 K4 臂的 K / N、s_lo 与 s_hi、G5 的每一句。
fn report_geometry_sensitivity(output: &mut ResultLines, outcome: &StreamOutcome) {
    let pair = |summary: &CellSummary, arm: Arm| {
        (
            u128::from(summary.arm_distinct_keys[arm.slot()]),
            u128::from(summary.states),
        )
    };
    let mut fields = vec![ratio_sensitivity_fields(
        "r1",
        (
            u128::from(outcome.union.distinct_unit_contents),
            u128::from(outcome.union.per_state_unit_checks),
        ),
        (
            u128::from(outcome.quick.distinct_unit_contents),
            u128::from(outcome.quick.per_state_unit_checks),
        ),
    )];
    for arm in [
        Arm::RecoveryPair,
        Arm::WalkUnits,
        Arm::AllUnits,
        Arm::FullReadSet,
    ] {
        fields.push(ratio_sensitivity_fields(
            &format!("{}_keys_over_states", arm.label()),
            pair(&outcome.union, arm),
            pair(&outcome.quick, arm),
        ));
    }
    for arm in ARMS {
        fields.push(format!(
            "{}_inconsistent_d1={} {}_inconsistent_quick={}",
            arm.label(),
            outcome.union.arm_inconsistent[arm.slot()],
            arm.label(),
            outcome.quick.arm_inconsistent[arm.slot()],
        ));
    }
    fields.push(share_sensitivity_fields(
        "unit_share",
        outcome.weighted,
        unweighted_unit_share(&outcome.quick),
    ));
    for arm in [Arm::WalkUnits, Arm::AllUnits, Arm::FullReadSet] {
        let sampled_parts = device_memory_parts(&outcome.union, outcome.write_table_length, arm);
        let quick = device_memory_parts(&outcome.quick, outcome.write_table_length, arm);
        let extrapolate = |parts: &DeviceMemoryParts, states: u64| {
            parts
                .linear(u128::from(states), EXTRAPOLATED_STATES)
                .total()
        };
        fields.push(capacity_sensitivity_fields(
            &format!("g5_{}_q5b_largest_card", arm.label()),
            extrapolate(&sampled_parts, outcome.union.states),
            extrapolate(&quick, outcome.quick.states),
            LARGEST_CARD_MEBIBYTES * MEBIBYTE,
        ));
        fields.push(capacity_sensitivity_fields(
            &format!("g5_{}_q5b_five_cards_total", arm.label()),
            extrapolate(&sampled_parts, outcome.union.states),
            extrapolate(&quick, outcome.quick.states),
            FIVE_CARDS_MEBIBYTES * MEBIBYTE,
        ));
        fields.push(format!(
            "g5_{}_bytes_per_state_d1={} g5_{}_bytes_per_state_quick={}",
            arm.label(),
            ratio_text(sampled_parts.total(), u128::from(outcome.union.states)),
            arm.label(),
            ratio_text(quick.total(), u128::from(outcome.quick.states)),
        ));
    }
    output.line(format!(
        "name=geometry_sensitivity knob=expansion stream={} {}",
        outcome.name.label(),
        fields.join(" ")
    ));
}

// ───────────────────────────── 判定行（第六节各行，5.5 第一段的够判规则） ─────────────────────────────

/// 每行各报各的判定，合取留给人（第六节开头）。退出码：S5 在产物那一趟没过就 3。
fn report_verdicts(
    output: &mut ResultLines,
    outcomes: &[&StreamOutcome],
    discriminating: &[Vec<(Arm, bool)>],
    timing_control_holds: bool,
    thread_lines_identical: bool,
) -> i32 {
    let mut determinism_clause_holds = true;
    let mut unit_check_fields = Vec::new();
    let mut recovery_reuse_fields = Vec::new();
    let mut time_split_fields = Vec::new();
    let mut checker_reuse_fields = Vec::new();
    let mut device_memory_fields = Vec::new();
    let found = |stream_index: usize, arm: Arm| {
        discriminating[stream_index]
            .iter()
            .any(|(candidate, has_one)| *candidate == arm && *has_one)
    };
    for (stream_index, outcome) in outcomes.iter().enumerate() {
        let name = outcome.name.label();
        let union = &outcome.union;
        for summary in [
            &outcome.small,
            &outcome.stride,
            &outcome.union,
            &outcome.quick,
        ] {
            determinism_clause_holds &= summary.consult_report_differs == 0
                && summary.ignore_report_differs == 0
                && summary.verdicts_differ == 0;
        }
        let pc1 = outcome.pc1.map_or(
            "not_in_this_stream",
            |(distinct_unchanged, checks_grew_exactly, _positions)| {
                bool_text(distinct_unchanged && checks_grew_exactly)
            },
        );
        unit_check_fields.push(format!(
            "{name}_r1_at_or_above_half_d1_estimate={} {name}_enough_to_judge=false {name}_v7_every_state_without_unit_reads={} {name}_content_collisions_zero={} {name}_pc1_holds={pc1}",
            bool_text(reaches_half(u128::from(union.distinct_unit_contents), u128::from(union.per_state_unit_checks))),
            bool_text(union.per_state_unit_checks == 0),
            bool_text(union.content_collisions == 0),
        ));
        for arm in [Arm::RecoveryConsult, Arm::RecoveryIgnore] {
            let slot = arm.slot();
            let flips = union.arm_inconsistent[slot] >= 1;
            recovery_reuse_fields.push(format!(
                "{name}_{arm}_inconsistent_zero={} {name}_{arm}_flips_to_every_state_rerun={} {name}_{arm}_enough_to_judge={} {name}_{arm}_hits_positive={} {name}_{arm}_pc2b_constant_key_inconsistent={} {name}_{arm}_pc2c_found_a_position={}",
                bool_text(!flips),
                bool_text(flips),
                bool_text(flips),
                bool_text(union.arm_hits[slot] > 0),
                bool_text(outcome.small.arm_constant_key_inconsistent[slot] >= 1),
                bool_text(found(stream_index, arm)),
                arm = arm.label(),
            ));
        }
        recovery_reuse_fields.push(format!(
            "{name}_k2_pair_keys_at_or_above_half_d1_estimate={} {name}_k2_pair_enough_to_judge=false",
            bool_text(reaches_half(
                u128::from(union.arm_distinct_keys[Arm::RecoveryPair.slot()]),
                u128::from(union.states)
            )),
        ));
        let share_verdict = ShareVerdict::of(outcome.weighted, threshold_half());
        let replay_share_over_one_percent =
            union.states_replaying_longer_than_the_checker * 100 > union.states;
        let shorter_share_over_one_percent =
            union.states_shorter_than_their_segments * 100 > union.states;
        time_split_fields.push(format!(
            "{name}_verdict={} {name}_enough_to_judge={} {name}_v3_segments_voided={} {name}_v3_replay_voided={}",
            share_verdict.label(),
            bool_text(share_is_enough_to_judge(outcome.weighted) && timing_control_holds),
            bool_text(shorter_share_over_one_percent),
            bool_text(replay_share_over_one_percent),
        ));
        for arm in [Arm::WalkUnits, Arm::AllUnits, Arm::FullReadSet] {
            let slot = arm.slot();
            let flips = union.arm_inconsistent[slot] >= 1;
            let constant_key_separates = union.arm_constant_key_inconsistent[slot] >= 1
                || union.arm_differing_pair[slot].is_some();
            checker_reuse_fields.push(format!(
                "{name}_{arm}_inconsistent_zero={} {name}_{arm}_flips_cannot_reuse={} {name}_{arm}_enough_to_judge={} {name}_{arm}_hits_positive={} {name}_{arm}_pc4b_separates={} {name}_{arm}_pc4c_found_a_position={} {name}_{arm}_keys_at_or_above_half_d1_estimate={}",
                bool_text(!flips),
                bool_text(flips),
                bool_text(flips),
                bool_text(union.arm_hits[slot] > 0),
                bool_text(constant_key_separates),
                bool_text(found(stream_index, arm)),
                bool_text(reaches_half(u128::from(union.arm_distinct_keys[slot]), u128::from(union.states))),
                arm = arm.label(),
            ));
        }
        checker_reuse_fields.push(format!(
            "{name}_s6_walk_reads_inside_normal_reads={}",
            bool_text(union.walk_reads_outside_normal_reads == 0)
        ));
        let per_state_bytes = mask_bytes_per_state(outcome.write_table_length)
            + u128::from(KEYS_STORED_PER_STATE * KEY_BYTES);
        let at_extrapolation = per_state_bytes * EXTRAPOLATED_STATES;
        device_memory_fields.push(format!(
            "{name}_per_state_parts_bytes={per_state_bytes} {name}_one_shot_per_state_parts_exceed_largest_card={} {name}_one_shot_per_state_parts_exceed_five_cards={}",
            bool_text(at_extrapolation > LARGEST_CARD_MEBIBYTES * MEBIBYTE),
            bool_text(at_extrapolation > FIVE_CARDS_MEBIBYTES * MEBIBYTE),
        ));
    }
    output.line(format!(
        "name=verdict scope=segment_one row=g1 {}",
        unit_check_fields.join(" ")
    ));
    output.line(format!(
        "name=verdict scope=segment_one row=g2 {}",
        recovery_reuse_fields.join(" ")
    ));
    output.line(format!(
        "name=verdict scope=segment_one row=g3 {} pc3_holds={} v6_threads_result_lines_identical={}",
        time_split_fields.join(" "),
        bool_text(timing_control_holds),
        bool_text(thread_lines_identical),
    ));
    output.line(format!(
        "name=verdict scope=segment_one row=g4 {}",
        checker_reuse_fields.join(" ")
    ));
    output.line(format!(
        "name=verdict scope=segment_one row=g5 {}",
        device_memory_fields.join(" ")
    ));
    output.line(format!(
        "name=verdict scope=segment_one row=s5 clause_S5_holds={}",
        bool_text(determinism_clause_holds)
    ));
    if determinism_clause_holds {
        0
    } else {
        3
    }
}

// ───────────────────────────── 模式 feasibility：可行性档（用户 2026-09-27 定，登记 12.2） ─────────────────────────────

/// 可行性档里 26 写段取段内序号最小的这么多个状态（用户 2026-09-27 定，登记 12.2）。
const FEASIBILITY_HEAD_STATES: u64 = 4_096;

/// 取样段里 26 写段的段头：段内序号 0 到 4095（按全量计划的序号）。
fn strided_segment_heads(stream: &PreparedStream, plan: &LocalPlan<'_>) -> Vec<u64> {
    let mut ordinals = Vec::new();
    for (segment_index, segment) in stream.segments.iter().enumerate() {
        if is_sampled_segment(stream.name, segment_index) && segment.len() == STRIDED_SEGMENT_WRITES
        {
            let first = plan.first_ordinal_by_segment[segment_index];
            let count = plan.count_by_segment[segment_index].min(FEASIBILITY_HEAD_STATES);
            ordinals.extend(first..first + count);
        }
    }
    ordinals
}

/// S4 在 26 写段段头上的形态：crates 按段枚举要把整段 1.5 亿个跑完，这里改用 crates 的单状态入口
/// `evaluate_state_for_versions`，在装置给的同一批持久集合上逐个评，计数与装置照跑的逐项比。交回相不相同。
fn report_per_state_crates_comparison(
    output: &mut ResultLines,
    label: &str,
    context: &EvaluationContext<'_>,
    plan: &LocalPlan<'_>,
    ordinals: &[u64],
    threads: usize,
) -> bool {
    let device = run_cell(&CellRun {
        label: format!("{label}-baseline"),
        context,
        plan,
        ordinals,
        control_ordinals: &BTreeSet::new(),
        busy_zone: None,
        threads,
        depth: EvaluationDepth::BaselineOnly,
    });
    let chunk = ordinals.len().div_ceil(threads.max(1)).max(1);
    let merged = Mutex::new(Layer0Tally::default());
    std::thread::scope(|scope| {
        for part in ordinals.chunks(chunk) {
            let merged = &merged;
            scope.spawn(move || {
                let mut tally = Layer0Tally::default();
                for ordinal in part {
                    let _report = singlefs_harness::crash::evaluate_state_for_versions(
                        &context.stream.base,
                        &context.table.writes,
                        plan.persisted_of(*ordinal),
                        context.stream.judged_root_index,
                        &context.stream.versions,
                        &mut tally,
                    );
                }
                add_tally(&mut merged.lock().expect("并计数的锁"), &tally);
            });
        }
    });
    let crates_tally = merged.into_inner().expect("并计数的锁");
    let device_text = tally_text(&device.tally);
    let crates_text = tally_text(&crates_tally);
    let holds = device_text == crates_text;
    output.line(format!(
        "name=stop_clause_crates_per_state cell={label} states={} clause_S4_holds={}",
        ordinals.len(),
        bool_text(holds)
    ));
    output.line(format!(
        "name=stop_clause_tally cell={label} side=device {device_text}"
    ));
    output.line(format!(
        "name=stop_clause_tally cell={label} side=crates {crates_text}"
    ));
    holds
}

/// 可行性档（用户 2026-09-27 定）：第一条流 D1-small、两条流的 D1-quick、两个 26 写段各取段头 4096 个状态；
/// 停机条款、阳性对照与 crates 的对拍都只在这个小域上做。G 行一律不判（只跑了可行性档）。
#[allow(
    clippy::too_many_lines,
    reason = "可行性档的全部格按登记 12.2 的次序排下来：S1、S2、锚点、与 crates 对拍、两遍对照、各格结果行、阳性对照、判定行"
)]
fn run_feasibility(output: &mut ResultLines) -> i32 {
    let threads = threads_from_environment();
    output.line(format!(
        "name=config experiment=E161 mode=feasibility threads={threads} registration=research/prompts/e161-preregistration.md domain=first_small_37+quick_54_84+strided_segment_heads_4096"
    ));
    let constants = local_constants_against_the_crates();
    let constants_clause_holds = constants
        .iter()
        .all(|(_name, local, crates)| local == crates);
    let constant_fields: Vec<String> = constants
        .iter()
        .map(|(name, local, crates)| format!("{name}={local}/{crates}"))
        .collect();
    output.line(format!(
        "name=stop_clause_constants {} clause_S1_holds={}",
        constant_fields.join(" "),
        bool_text(constants_clause_holds)
    ));
    let first = prepare_first_stream();
    let second = prepare_second_stream();
    let (bases_identical, prefix_identical) = shared_prefix_is_identical(&first, &second);
    let mut streams_clause_holds = bases_identical && prefix_identical;
    for stream in [&first, &second] {
        let check = check_stream_shape(stream);
        streams_clause_holds &=
            check.segment_lengths_match && check.writes_match && check.roots_match;
        output.line(format!(
            "name=stop_clause_stream stream={} writes={} segments={} roots={} segment_lengths_match={} writes_match={} roots_match={}",
            stream.name.label(),
            stream.writes.len(),
            stream.segments.len(),
            stream.root_write_count,
            bool_text(check.segment_lengths_match),
            bool_text(check.writes_match),
            bool_text(check.roots_match),
        ));
    }
    output.line(format!(
        "name=stop_clause_shared_prefix bases_identical={} first_nine_segments_identical={} clause_S2_holds={}",
        bool_text(bases_identical),
        bool_text(prefix_identical),
        bool_text(streams_clause_holds)
    ));
    let mut anchors_hold = capacity_anchors_hold();
    let mut enumeration_clause_holds = true;
    let mut tally_clause_holds = true;
    let mut determinism_clause_holds = true;
    let mut walk_subset_clause_holds = true;
    let mut positive_controls: Vec<(String, bool)> = Vec::new();
    for stream in [&first, &second] {
        let name = stream.name.label();
        let table = TornWriteTable::of(stream);
        anchors_hold &= report_stream_anchors(output, stream, &table);
        let context = EvaluationContext::new(stream, &table);
        let full = LocalPlan::new(
            stream,
            &table,
            domain_expansions(stream, LocalExpansion::EveryCombination),
        );
        let quick = LocalPlan::new(
            stream,
            &table,
            domain_expansions(
                stream,
                LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll,
            ),
        );
        let small = match stream.name {
            StreamName::First => small_domain_ordinals(stream, &full),
            StreamName::Second => Vec::new(),
        };
        let heads = strided_segment_heads(stream, &full);
        let quick_ordinals = every_ordinal(stream, &quick);
        if !small.is_empty() {
            let small_segments: BTreeSet<usize> = small
                .iter()
                .map(|ordinal| full.segment_of(*ordinal))
                .collect();
            let (small_enumeration_holds, small_tally_holds) = report_crates_comparison(
                output,
                &format!("{name}_small"),
                &context,
                &full,
                &small,
                &|segment_index| {
                    if small_segments.contains(&segment_index) {
                        Layer0SegmentExpansion::EveryProperSubset
                    } else {
                        Layer0SegmentExpansion::NotExpanded
                    }
                },
                threads,
            );
            enumeration_clause_holds &= small_enumeration_holds;
            tally_clause_holds &= small_tally_holds;
        }
        let (quick_enumeration_holds, quick_tally_holds) = report_crates_comparison(
            output,
            &format!("{name}_quick"),
            &context,
            &quick,
            &quick_ordinals,
            &|segment_index| quick.expansion_by_segment[segment_index].crates_expansion(),
            threads,
        );
        enumeration_clause_holds &= quick_enumeration_holds;
        tally_clause_holds &= quick_tally_holds;
        tally_clause_holds &= report_per_state_crates_comparison(
            output,
            &format!("{name}_segment_head"),
            &context,
            &full,
            &heads,
            threads,
        );
        let control: BTreeSet<u64> = small.iter().chain(&heads).copied().collect();
        let (control_determinism_holds, control_walk_subset_holds) = report_control_double_run(
            output,
            &format!("{name}_small_and_head"),
            &context,
            &full,
            &control,
            threads,
        );
        determinism_clause_holds &= control_determinism_holds;
        walk_subset_clause_holds &= control_walk_subset_holds;
        let mut control_data: Vec<ControlStateData> = Vec::new();
        let mut cells: Vec<(&str, &[u64], &LocalPlan<'_>, bool)> = vec![
            ("segment_head", &heads, &full, true),
            ("quick", &quick_ordinals, &quick, false),
        ];
        if !small.is_empty() {
            cells.insert(0, ("small", &small, &full, true));
        }
        for (domain, ordinals, plan, keeps_control) in cells {
            let label = stream_label(stream.name, domain);
            let control_ordinals: BTreeSet<u64> = if keeps_control {
                ordinals.iter().copied().collect()
            } else {
                BTreeSet::new()
            };
            let mut totals = run_cell(&CellRun {
                label: label.clone(),
                context: &context,
                plan,
                ordinals,
                control_ordinals: &control_ordinals,
                busy_zone: None,
                threads,
                depth: EvaluationDepth::FourSteps,
            });
            let stores = std::mem::take(&mut totals.arm_stores);
            let strided: BTreeSet<usize> = BTreeSet::new();
            let summary = report_cell(
                output,
                &label,
                &totals,
                stores,
                &TrajectoryIntervals::of(plan, ordinals, &strided),
                None,
            );
            determinism_clause_holds &= summary.consult_report_differs == 0
                && summary.ignore_report_differs == 0
                && summary.verdicts_differ == 0;
            for arm in [Arm::RecoveryConsult, Arm::RecoveryIgnore, Arm::FullReadSet] {
                positive_controls.push((
                    format!("{label}_{}_constant_key_separates", arm.label()),
                    summary.arm_constant_key_inconsistent[arm.slot()] >= 1
                        || summary.arm_differing_pair[arm.slot()].is_some(),
                ));
            }
            if domain == "small" {
                let (distinct_unchanged, checks_grew_exactly, positions) =
                    refeed_every_write_persisted(
                        &context,
                        &full,
                        &totals.tables,
                        totals.whole_unit_positions,
                    );
                output.line(format!(
                    "name=positive_control_refeed cell={label} refed_state_unit_positions={positions} distinct_unit_contents_unchanged={} per_state_checks_grew_by_exactly_its_positions={} pc1_holds={}",
                    bool_text(distinct_unchanged),
                    bool_text(checks_grew_exactly),
                    bool_text(distinct_unchanged && checks_grew_exactly),
                ));
                positive_controls.push((
                    format!("{label}_pc1"),
                    distinct_unchanged && checks_grew_exactly,
                ));
            }
            if domain == "segment_head" {
                let holds = report_device_memory_control(
                    output,
                    &label,
                    &mut totals.tables,
                    &summary,
                    table.mask_width(),
                );
                positive_controls.push((format!("{label}_pc5"), holds));
            }
            control_data.extend(totals.control);
        }
        control_data.sort_by_key(|data| data.ordinal);
        for (arm, found) in report_discriminating_positions(
            output,
            &format!("{name}_small_and_head"),
            &control_data,
            threads,
        ) {
            positive_controls.push((
                format!("{name}_{}_pc2c_pc4c_found_a_position", arm.label()),
                found,
            ));
        }
    }
    let timing_control_holds = report_timing_control(output, &second, threads);
    positive_controls.push(("second_segment_14_pc3".to_string(), timing_control_holds));
    let control_fields: Vec<String> = positive_controls
        .iter()
        .map(|(name, holds)| format!("{name}={}", bool_text(*holds)))
        .collect();
    output.line(format!(
        "name=verdict scope=feasibility clause_S1_holds={} clause_S2_holds={} clause_S3_holds={} clause_S4_holds={} clause_S5_holds={} clause_S6_holds={} anchors_a1_to_a7_hold={} {} g1=not_judged_feasibility_only g2=not_judged_feasibility_only g3=not_judged_feasibility_only g4=not_judged_feasibility_only g5=not_judged_feasibility_only",
        bool_text(constants_clause_holds),
        bool_text(streams_clause_holds),
        bool_text(enumeration_clause_holds),
        bool_text(tally_clause_holds),
        bool_text(determinism_clause_holds),
        bool_text(walk_subset_clause_holds),
        bool_text(anchors_hold),
        control_fields.join(" "),
    ));
    if constants_clause_holds
        && streams_clause_holds
        && enumeration_clause_holds
        && tally_clause_holds
        && determinism_clause_holds
        && anchors_hold
    {
        0
    } else {
        3
    }
}

// e161-append-point

#[cfg(test)]
mod tests {
    use super::*;

    /// 第一条流的 D1-small（37 个状态）跑一遍 5.1 的四件事；各测试共用这一份搭建的写法。
    fn first_stream_small_cell(
        threads: usize,
        busy_zone: Option<TimingZone>,
    ) -> (PreparedStream, CellTotals, Vec<u64>) {
        let stream = prepare_first_stream();
        let totals = {
            let table = TornWriteTable::of(&stream);
            let context = EvaluationContext::new(&stream, &table);
            let plan = LocalPlan::new(
                &stream,
                &table,
                domain_expansions(&stream, LocalExpansion::EveryCombination),
            );
            let small = small_domain_ordinals(&stream, &plan);
            let control: BTreeSet<u64> = small.iter().copied().collect();
            run_cell(&CellRun {
                label: "test-first-small".to_string(),
                context: &context,
                plan: &plan,
                ordinals: &small,
                control_ordinals: &control,
                busy_zone,
                threads,
                depth: EvaluationDepth::FourSteps,
            })
        };
        let small = {
            let table = TornWriteTable::of(&stream);
            let plan = LocalPlan::new(
                &stream,
                &table,
                domain_expansions(&stream, LocalExpansion::EveryCombination),
            );
            small_domain_ordinals(&stream, &plan)
        };
        (stream, totals, small)
    }

    #[test]
    fn every_local_constant_equals_its_crates_counterpart() {
        let constants = local_constants_against_the_crates();
        assert_eq!(constants.len(), 14, "S1 列了十四个常量");
        for (name, local, crates) in constants {
            assert_eq!(local, crates, "S1：{name} 本地 {local}，crates {crates}");
        }
        assert_eq!(
            UNIT_AREA_START_BYTES, 822_083_584,
            "单元区起点 50176 × 16384"
        );
        assert_eq!(RING_START_BYTES, 16_777_216, "环起点 1024 × 16384");
    }

    #[test]
    fn first_stream_plan_counts_match_the_formula_the_crates_and_the_anchor() {
        let stream = prepare_first_stream();
        let table = TornWriteTable::of(&stream);
        let checks = segment_count_checks_up_to(&stream, &table, 1 << 20);
        let counted: Vec<u64> = checks
            .iter()
            .map(|check| check.counted_by_the_plan)
            .collect();
        assert_eq!(
            counted, ANCHOR_FIRST_STREAM_SEGMENT_STATES,
            "A2：装置计划逐段数"
        );
        for check in &checks {
            assert_eq!(
                check.counted_by_the_plan, check.counted_by_the_crates,
                "A1：段 {} 装置对 crates",
                check.segment_index
            );
            assert_eq!(
                check.counted_by_the_crates, check.counted_by_the_formula,
                "A1：段 {} crates 对公式",
                check.segment_index
            );
        }
        assert_eq!(
            table.mask_width(),
            49,
            "A5：41 次写 + 8 份撕裂镜像 + 0 次重放"
        );
        assert_eq!(
            formula_segment_state_count(26, 2),
            150_994_943,
            "3² · 2²⁴ − 1"
        );
    }

    #[test]
    fn stride_and_quick_domains_have_the_registered_sizes() {
        let stream = prepare_first_stream();
        let table = TornWriteTable::of(&stream);
        let full = LocalPlan::new(
            &stream,
            &table,
            domain_expansions(&stream, LocalExpansion::EveryCombination),
        );
        let quick = LocalPlan::new(
            &stream,
            &table,
            domain_expansions(
                &stream,
                LocalExpansion::InPlaceCombinationsWithUnitWritesNoneOrAll,
            ),
        );
        assert_eq!(
            small_domain_ordinals(&stream, &full).len(),
            37,
            "A4：D1-small 第一条流"
        );
        assert_eq!(
            stride_domain_ordinals(&stream, &full).len(),
            65_338,
            "A4：D1-stride 每个 26 写段"
        );
        assert_eq!(
            every_ordinal(&stream, &quick).len(),
            54,
            "A4：D1-quick 第一条流"
        );
    }

    #[test]
    fn recorded_recovery_answers_exactly_like_the_baseline_on_the_first_stream() {
        let (_stream, totals, _small) = first_stream_small_cell(2, None);
        assert_eq!(totals.states(), 37, "D1-small 第一条流 37 个状态");
        assert_eq!(
            totals.consult_report_differs, 0,
            "S5：记录跑看 journal 那一遍与照跑逐项相同"
        );
        assert_eq!(totals.ignore_report_differs, 0, "S5：不看 journal 那一遍");
        assert_eq!(totals.verdicts_differ, 0, "S5：checker 正常那一遍");
        assert_eq!(
            totals.tally.file_read_states, 12,
            "4.1：读出文件的 12 个状态都在 D1-small 里"
        );
    }

    #[test]
    fn constant_keys_separate_recovery_outcomes_and_verdicts_on_the_first_stream() {
        let (_stream, mut totals, small) = first_stream_small_cell(2, None);
        let stream = prepare_first_stream();
        let table = TornWriteTable::of(&stream);
        let plan = LocalPlan::new(
            &stream,
            &table,
            domain_expansions(&stream, LocalExpansion::EveryCombination),
        );
        let intervals = TrajectoryIntervals::of(&plan, &small, &BTreeSet::new());
        let stores = std::mem::take(&mut totals.arm_stores);
        let mut output = ResultLines::new();
        let summary = report_cell(
            &mut output,
            "test-first-small",
            &totals,
            stores,
            &intervals,
            None,
        );
        assert!(
            summary.arm_constant_key_inconsistent[Arm::RecoveryConsult.slot()] >= 1,
            "PC2b：键换成常数时看 journal 那一遍不一致数 ≥ 1"
        );
        assert!(
            summary.arm_constant_key_inconsistent[Arm::FullReadSet.slot()] >= 1,
            "PC4b：键换成常数时判定不一致数 ≥ 1"
        );
        assert_eq!(
            summary.arm_inconsistent[Arm::RecoveryConsult.slot()],
            0,
            "按读集复用在这 37 个状态上没有不一致"
        );
    }

    #[test]
    fn refeeding_the_every_write_persisted_state_keeps_distinct_unit_contents() {
        let (stream, totals, _small) = first_stream_small_cell(2, None);
        let table = TornWriteTable::of(&stream);
        let context = EvaluationContext::new(&stream, &table);
        let plan = LocalPlan::new(
            &stream,
            &table,
            domain_expansions(&stream, LocalExpansion::EveryCombination),
        );
        let (distinct_unchanged, checks_grew_exactly, positions) = refeed_every_write_persisted(
            &context,
            &plan,
            &totals.tables,
            totals.whole_unit_positions,
        );
        assert!(distinct_unchanged, "PC1：去重臂的不同内容数不变");
        assert!(
            checks_grew_exactly,
            "PC1：逐状态臂恰好加上它的不同单元位置数"
        );
        assert!(positions > 0, "全部持久那个状态读到了单元");
    }

    #[test]
    fn busy_wait_in_the_consult_zone_lands_in_the_consult_zone() {
        let (_stream, quiet, _small) = first_stream_small_cell(1, None);
        let (_busy_stream, busy, _busy_small) =
            first_stream_small_cell(1, Some(TimingZone::RecoveryConsult));
        let zone_total = |totals: &CellTotals, zone: usize| -> u128 {
            totals
                .timing_by_segment
                .values()
                .map(|sums| sums.zone_nanoseconds[zone])
                .sum()
        };
        let added = zone_total(&busy, 0).saturating_sub(zone_total(&quiet, 0));
        assert!(
            added >= 37 * TIMING_CONTROL_BUSY_WAIT.as_nanos() * 9 / 10,
            "PC3 的缩小版：看 journal 那一区多出 37 × 2 ms 的九成以上，实际 {added} ns"
        );
    }

    #[test]
    fn device_memory_parts_grow_by_the_synthetic_amounts() {
        let (stream, mut totals, small) = first_stream_small_cell(2, None);
        let table = TornWriteTable::of(&stream);
        let plan = LocalPlan::new(
            &stream,
            &table,
            domain_expansions(&stream, LocalExpansion::EveryCombination),
        );
        let intervals = TrajectoryIntervals::of(&plan, &small, &BTreeSet::new());
        let stores = std::mem::take(&mut totals.arm_stores);
        let mut output = ResultLines::new();
        let summary = report_cell(
            &mut output,
            "test-first-small",
            &totals,
            stores,
            &intervals,
            None,
        );
        assert!(
            report_device_memory_control(
                &mut output,
                "test-first-small",
                &mut totals.tables,
                &summary,
                table.mask_width()
            ),
            "PC5：P1 加 32768、P3 加 7 × 21、P2 加 ⌈W/8⌉、P4 加 48"
        );
        assert_eq!(mask_bytes_per_state(49), 7, "⌈49 / 8⌉");
    }

    #[test]
    fn device_tally_equals_the_crates_tally_on_the_first_stream_small_domain() {
        let stream = prepare_first_stream();
        let table = TornWriteTable::of(&stream);
        let context = EvaluationContext::new(&stream, &table);
        let plan = LocalPlan::new(
            &stream,
            &table,
            domain_expansions(&stream, LocalExpansion::EveryCombination),
        );
        let small = small_domain_ordinals(&stream, &plan);
        let small_segments: BTreeSet<usize> = small
            .iter()
            .map(|ordinal| plan.segment_of(*ordinal))
            .collect();
        let mut output = ResultLines::new();
        let (enumeration_clause_holds, tally_clause_holds) = report_crates_comparison(
            &mut output,
            "test-first-small",
            &context,
            &plan,
            &small,
            &|segment_index| {
                if small_segments.contains(&segment_index) {
                    Layer0SegmentExpansion::EveryProperSubset
                } else {
                    Layer0SegmentExpansion::NotExpanded
                }
            },
            2,
        );
        assert!(enumeration_clause_holds, "S3：持久集合与写表逐个相同");
        assert!(tally_clause_holds, "S4：装置照跑的计数与 crates 逐项相同");
    }

    #[test]
    fn result_lines_count_every_line_they_print() {
        let mut output = ResultLines::new();
        output.line("name=test first".to_string());
        output.line("name=test second".to_string());
        output.line("name=test third".to_string());
        assert_eq!(output.emitted, 3, "V4：收尾计数行数的是真打出去的行");
    }

    #[test]
    fn content_first_appearance_does_not_depend_on_the_merge_order() {
        let stream = prepare_first_stream();
        let table = TornWriteTable::of(&stream);
        let context = EvaluationContext::new(&stream, &table);
        let plan = LocalPlan::new(
            &stream,
            &table,
            domain_expansions(&stream, LocalExpansion::EveryCombination),
        );
        let stride: Vec<u64> = stride_domain_ordinals(&stream, &plan)
            .into_iter()
            .take(STATES_PER_SLICE * 3)
            .collect();
        let intervals = TrajectoryIntervals {
            starts: stride.iter().step_by(16).copied().collect(),
        };
        let lines_with = |threads: usize| {
            let mut totals = run_cell(&CellRun {
                label: format!("test-merge-order-{threads}"),
                context: &context,
                plan: &plan,
                ordinals: &stride,
                control_ordinals: &BTreeSet::new(),
                busy_zone: None,
                threads,
                depth: EvaluationDepth::FourSteps,
            });
            let stores = std::mem::take(&mut totals.arm_stores);
            let mut output = ResultLines::new();
            report_cell(
                &mut output,
                "test-merge-order",
                &totals,
                stores,
                &intervals,
                None,
            )
            .lines_without_timing
        };
        assert_eq!(
            lines_with(1),
            lines_with(4),
            "V6：线程数 1 与 4 的结果行（去掉计时）逐字相同"
        );
    }

    #[test]
    fn spilled_records_come_back_in_the_same_order_as_in_memory() {
        let records: Vec<ArmRecord> = (0..1_000u64)
            .map(|index| ArmRecord {
                key: Digest128(u128::from((index * 7_919) % 97)),
                ordinal: 1_000 - index,
                outcome: Digest128(u128::from(index % 3)),
                elements: u32::try_from(index % 11).expect("小于 11"),
            })
            .collect();
        let mut in_memory = ArmRecordStore::new(usize::MAX);
        let mut spilled = ArmRecordStore::new(64);
        for record in &records {
            in_memory.push(*record);
            spilled.push(*record);
        }
        assert!(spilled.runs.len() >= 15, "每 64 条溢出一段");
        let mut first = Vec::new();
        in_memory.visit_sorted(&mut |record| first.push(record));
        let mut second = Vec::new();
        spilled.visit_sorted(&mut |record| second.push(record));
        assert_eq!(first.len(), 1_000, "一条不少");
        assert_eq!(first, second, "外排序与内存排序逐条相同");
    }

    #[test]
    fn digest_is_deterministic_and_separates_one_byte() {
        assert_eq!(
            digest_of_bytes(b"e161"),
            digest_of_bytes(b"e161"),
            "同样的字节同样的摘要"
        );
        assert_ne!(
            digest_of_bytes(b"e161"),
            digest_of_bytes(b"e162"),
            "差一个字节摘要不同"
        );
        assert_ne!(
            digest_of_bytes(&[0u8; 8]),
            digest_of_bytes(&[0u8; 9]),
            "长度进摘要"
        );
    }
}
