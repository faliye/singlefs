//! 验证包模块：crash、crash_state_enumeration、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器照 CPU 判 I-7.9 不带卸载记号那一支的丁（C717（回退下界上限的单窗口盲带）：丁，借 journal 判红）：
//! 锚在镜像最新根上的一圈窗口里的根判成立、而窗口定不住抬 F 那一刻的上限（去掉最旧那条之后非空有效根去重不足 4 条）时，
//! 按 journal 重建抬 F 那一刻的根环再判（池级 checker `crates/singlefs-checker/src/walk.rs` 的
//! `rollback_floor_against_the_journal_reconstruction_of_the_ring_at_the_raise`，扫描内核 `crash_judge_checker_scan.wgsl` 的
//! `c_root_in_the_ring_slot_at_the_raise`）。CPU 一侧单测那一对镜像只喂 I-7.9 的判法、不是整池镜像，喂不进 GPU；这里在新池新建文件那条流
//! （根环 txg 0、1、2 与新建文件那次发布 (1, 3)，3S = 24）的流尾逐段写三条抄 (1, 3) 的根，造三张整池镜像：
//! txg 25（回退下界 F 2、带卸载记号）盖掉 txg 1 那一槽，txg 26（F 2）盖掉 txg 2 那一槽，txg 27（F 抬到 `raised_floor`）盖掉 txg 3 那一槽。
//! 三条都落了的状态上抬 F 的是 txg 27：窗口（txg > 3）里之前的有效根是 25、26，上限 25，窗口判成立；两条是同一个状态，窗口定不住。
//! 重建抬 F 那一刻的环：区域 0 槽 0 住第 0 代根（txg 0 < F 2，无效），25、26 照旧，txg 3 那一槽被 27 盖掉、按 journal 里 txg 3 那一版
//! （实例 1、带提交标记）认出 txg 3；有效根 3、25、26，非空的只有 3，上限取最旧有效根 3。
//! 合法写者那张 F 抬到 3（成立）；坏写者那张 F 抬到 4（窗口里成立、重建的环上限 3，红）；坏写者那张再把 txg 3 那次发布的 journal 记录
//! 提交标记写成 0（journal 罩不到 txg 3，环重建不出，射程里判不了）。前两条根的状态上 txg 25 是带卸载记号的抬 F（0 → 2），
//! 它之前窗口里最新的根是 txg 3，成立；txg 26 不抬。
//! 三张逐状态比 CPU 与 GPU 的对拍字（违例位、判到位——射程里判不了的那一条判到位为 0——与恢复、oracle 那几个字）。
//! 跨实例那一支（C935（I-7.9 按 journal 重建时 u 属于另一个实例那一支没有用例））：被盖掉那一槽在抬 F 那一刻住的那一版 u 属于前一个实例时，
//! 要看抬 F 那条根的实例表里 u 那个实例的行 (i, T, W)，u < T 才认作 u 的根槽落了盘。这一支另起一条双实例的流：新池新建文件之后覆盖写一次
//! （实例 1、txg 4）、正常卸载（F 抬到 4，三次带卸载记号的空发布 txg 5、6、7）、关掉镜像重开可写挂载（实例 2：取号、写行、暖机，txg 8、9、10；
//! 实例 2 的实例表里实例 1 那一行 T = 7），流尾抄实例 2 最后那条根写几条根，造三张整池镜像：
//! txg 28（F 4）盖掉 txg 4 那一槽、txg 29 把 F 抬上去并盖掉 txg 5 那一槽——抬 F 那一刻那一槽住的是实例 1 中间那一版 txg 5（5 < T 7，认得出）；
//! 窗口（txg > 5）里之前的有效根 6、7、8、9、10、28 是同一个状态，上限 6、定不住；重建的环里有效根 5、6、7、8、9、10、28，非空的只有 5，上限 5。
//! 合法写者那张 F 抬到 5（成立），坏写者那张 F 抬到 6（窗口里成立、重建的环上限 5，红）；被盖的是实例 1 最后一版的两张抄 txg 28、29、30（F 4）、
//! txg 31 抬 F、盖掉 txg 7 那一槽：抬 F 那一刻那一槽住的可能是实例 1 的最后一版 txg 7（u = T，根槽可以没落），也可能是空着（落点式住进那一槽、
//! 比 7 小的正 txg 没有，那一槽不是槽 0），两种都算、各判一次（判决 `research/prompts/m3-close-b19-reason-r1-main-verification.md` 第二节 Q2「定」那一行）：
//! 住 txg 7 那一种上限 7，空着那一种上限 8。F 抬到 6 的那张两种都成立、判成立；F 抬到 8 的那张一种红一种成立、射程里判不了；
//! 跨实例那一支不看实例表的行（当成同一实例）时只认 txg 7 那一种，F 8 那张判红，CPU 与 GPU 有一边这样判就对不上。
//! 各张会判成什么都是推的，用例先钉 CPU 的判定，推错了前提断言先红并逐状态打出 CPU 判成什么。
//! 要本机一张空闲显存够的 NVIDIA 独显：一张独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
//! （`gpu_unit_checks::gpu_test_start`）。要 `gpu` 特性。
#![cfg(feature = "gpu")]

mod common;

use common::{
    build_pool, file_content, geometry, parameters, FILE_BYTES, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::mount::{mount_writable, unmount, ShadowLedger, Unmounted};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolVersion, PoolWriter};
use singlefs_verification::crash::{
    crash_point_replay_plan_state_count, CrashPointReplaySegmentExpansion,
};
use singlefs_verification::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_verification::crash_judge_gpu::{
    cross_check_words_agree_in_the_cross_check, cross_check_words_of_cpu_judgement,
    cross_check_words_of_gpu_verdict, CrossCheckWords, VERDICT_WORDS,
};
use singlefs_verification::crash_judge_tables::{build_judge_tables, JudgeFlow};
use singlefs_verification::crash_state_enumeration::{
    sample_crash_states_on_the_cpu, CrashPointReplayItem,
};
use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
use singlefs_verification::memory_pool::{
    writes_and_segments_with_stream_indexes, writes_and_segments_with_stream_indexes_and_entries,
    MemoryPool, PublishedVersion, RetainedWrite, WrittenContents,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::{RecordedEntrySpan, RecordedPublishEntry};
use std::sync::Arc;

/// 这份测试文件的仓内路径：崩溃点重放取样过库的设点文件。
const THIS_FILE: &str =
    "crates/singlefs-verification/tests/crash_judge_gpu_rollback_floor_ceiling_rebuilt_from_the_journal.rs";
/// 段长不超过这个数的段逐个真子集展开（与 GPU 对拍那一份的默认展开上限同一档）。
const EXPAND_UP_TO: usize = 8;

/// 根记录字段表（D22（单元原子性怎么合成） 已定项 7 的根记录布局）：flags（位 0 是卸载记号）、实例代号、checkpoint_txg、回退下界 F、整槽校验和。
const ROOT_RECORD_FLAGS_OFFSET: usize = 20;
const ROOT_RECORD_INSTANCE_OFFSET: usize = 24;
const ROOT_RECORD_CHECKPOINT_TXG_OFFSET: usize = 28;
const ROOT_RECORD_ROLLBACK_FLOOR_OFFSET: usize = 130;
const ROOT_RECORD_CHECKSUM_FIELD_OFFSET: usize = 138;
const ROOT_RECORD_CHECKSUM_BYTES: usize = 32;

/// journal 记录字段表（D23（journal 的角色与格式） 已定项 17）：实例代号、计数器（6 字节）、checkpoint_txg、头校验和（罩 4096 字节、
/// 字段按 0 参与）、提交标记、反向链、载荷校验和（罩点名项）；头 496 字节，点名项每条 56 字节。
const JOURNAL_RECORD_NAMED_ENTRY_COUNT_OFFSET: usize = 12;
const JOURNAL_RECORD_INSTANCE_OFFSET: usize = 16;
const JOURNAL_RECORD_COUNTER_OFFSET: usize = 20;
const JOURNAL_RECORD_COUNTER_BYTES: usize = 6;
const JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET: usize = 26;
const JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET: usize = 46;
const JOURNAL_RECORD_HEADER_CHECKSUM_BYTES: usize = 32;
const JOURNAL_RECORD_COMMIT_MARKER_OFFSET: usize = 86;
const JOURNAL_RECORD_BACK_CHAIN_OFFSET: usize = 91;
const JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET: usize = 95;
const JOURNAL_RECORD_HEADER_BYTES: usize = 496;
const JOURNAL_RECORD_NAMED_ENTRY_BYTES: usize = 56;
const JOURNAL_RECORD_CHECKSUMMED_BYTES: usize = 4096;
/// 提交标记字节取 1 是带提交标记、取 0 是不带（D23（journal 的角色与格式） 已定项 17；只许这两个值）。
const JOURNAL_RECORD_COMMIT_MARKER_PRESENT: u8 = 1;
const JOURNAL_RECORD_COMMIT_MARKER_ABSENT: u8 = 0;

/// 新池新建文件那条流的根环：每区 8 槽、三个区域，3S = 24（流尾那三条根的 txg 照它挑）。
const ROOT_RING_SLOT_COUNT_OF_THE_NEW_POOL: u64 = 24;
/// 新建文件那次发布的 (实例代号, checkpoint_txg)：流尾三条根都抄它，txg 27 盖掉的就是它那一槽。
const PUBLISH_ROOT_IDENTITY: (u32, u64) = (1, 3);
/// 流尾三条根：txg 25、26 的回退下界 F（25 那条带卸载记号，抬 F 0 → 2 走带记号那一支），txg 27 再抬 F。
const TXG_OF_THE_UNMOUNT_MARKED_ROOT: u64 = 25;
const TXG_OF_THE_ROOT_BEFORE_THE_RAISE: u64 = 26;
const TXG_OF_THE_ROOT_RAISING_THE_FLOOR: u64 = 27;
const FLOOR_BEFORE_THE_RAISE: u64 = 2;
/// 重建的抬 F 那一刻的环算出的上限：最旧有效根 txg 3（按 journal 认出的那一版）。合法写者把 F 抬到它，坏写者抬过它。
/// 坏写者抬到 4、不抬到更高：流尾直接从 txg 3 跳到 25，环里落点式住 txg 4–23 的那几槽读回全零，实例 1 在环里最旧的根是 txg 25
/// （txg 1、2、3 那几槽被 25、26、27 盖掉了），池级 checker 把这几槽并进上限（C961（全零根槽没并进 checker 的抬 F 上限），
/// `walk.rs` 的 `rollback_floor_ceiling_held_by_the_ring_slots_reading_back_all_zero`：能住的 txg 比抬 F 那个实例在环里最旧的根小的才并），
/// 住 txg 4 的那一槽把上限压在 4：F 越过 4 时先红在全零槽上、走不到按 journal 重建的环；F 4 不越过它，窗口里成立、重建的环上限 3，红在丁上。
const FLOOR_RAISED_TO_THE_CEILING_OF_THE_RECONSTRUCTED_RING: u64 = 3;
const FLOOR_RAISED_PAST_THE_CEILING_OF_THE_RECONSTRUCTED_RING: u64 = 4;

/// 双实例那条流（文件头「跨实例那一支」）：卸载的是实例 1，卸载之后重开可写挂载取到实例 2。
const UNMOUNTED_INSTANCE: u32 = 1;
const INSTANCE_MOUNTED_AFTER_THE_UNMOUNT: u32 = 2;
/// 双实例那条流里根槽写依次的 (实例代号, checkpoint_txg)：暖机两代、新建文件、覆盖写、卸载那一串三次（现行 txg 4 除以 3 余 1）、
/// 挂载取号写行暖机三次（同 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 回退那条流里挂载的三条根）。造流时逐条核它。
const ROOT_IDENTITIES_OF_THE_FLOW_THROUGH_THE_UNMOUNT_AND_THE_MOUNT_AFTER_IT: [(u32, u64); 10] = [
    (1, 1),
    (1, 2),
    (1, 3),
    (1, 4),
    (1, 5),
    (1, 6),
    (1, 7),
    (2, 8),
    (2, 9),
    (2, 10),
];
/// 正常卸载把 F 抬到现行那一版（覆盖写那一版）的 txg：实例 2 的根都带它，流尾抄的根不抬 F 时也带它。
const FLOOR_RAISED_BY_THE_UNMOUNT: u64 = 4;
/// 实例 1 中间那一版（卸载那一串里 txg 最小的那次空发布）：比实例 2 的实例表里实例 1 那一行的 T 小，根槽落了盘。
const INTERMEDIATE_TXG_OF_THE_UNMOUNTED_INSTANCE: u64 = 5;
/// 实例 1 的最后一版（卸载那一串最后一次空发布）：实例 2 的实例表里实例 1 那一行的 T。
const LAST_TXG_OF_THE_UNMOUNTED_INSTANCE: u64 = 7;
/// 抬 F 盖掉实例 1 中间那一版那一槽的两张：流尾先抄 txg 28（F 4，盖掉 txg 4 那一槽，抬 F 那一刻那一槽已住着它），再抄 txg 29 抬 F。
const TXGS_BEFORE_THE_RAISE_OVER_THE_INTERMEDIATE_VERSION: [u64; 1] = [28];
const TXG_OF_THE_RAISE_OVER_THE_INTERMEDIATE_VERSION: u64 = 29;
/// 重建的抬 F 那一刻的环（有效根 5、6、7、8、9、10、28，非空的只有实例 1 的 txg 5）算出的上限 5；窗口（txg > 5）里算出的是 6。
const FLOOR_RAISED_TO_THE_CEILING_OF_THE_RING_HOLDING_THE_INTERMEDIATE_VERSION: u64 = 5;
const FLOOR_RAISED_PAST_THE_CEILING_OF_THE_RING_HOLDING_THE_INTERMEDIATE_VERSION: u64 = 6;
/// 抬 F 盖掉实例 1 最后一版那一槽的两张：流尾先抄 txg 28、29、30（F 4，盖掉 txg 4、5、6 那三槽），再抄 txg 31 抬 F
/// （窗口（txg > 7）里算出的上限 8，窗口里成立而定不住；按 journal 重建的环那一槽住 txg 7 那一种上限 7、空着那一种上限 8）。
const TXGS_BEFORE_THE_RAISE_OVER_THE_LAST_VERSION: [u64; 3] = [28, 29, 30];
const TXG_OF_THE_RAISE_OVER_THE_LAST_VERSION: u64 = 31;
/// 两种可能的环都不越过的 F（不高于 7）：判成立。
const FLOOR_RAISED_WITHIN_BOTH_POSSIBLE_RINGS_OVER_THE_SLOT_OF_THE_LAST_VERSION: u64 = 6;
/// 只在空着那一种里不越过的 F（高于 7、不高于 8）：两种判得不一致，射程里判不了。
const FLOOR_RAISED_PAST_ONE_POSSIBLE_RING_OVER_THE_SLOT_OF_THE_LAST_VERSION: u64 = 8;

/// 一条流：mkfs 之后的基镜像、写表与段、判的那条根、模型认的版本。
struct RecordedFlow {
    name: &'static str,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
}

/// 新池新建文件那条流（与 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 的 `record_new_pool_file_creation` 同一个造法）。
fn record_new_pool_file_creation(tag: &str, name: &'static str) -> RecordedFlow {
    let pool = build_pool(tag);
    let operations = pool.retained_operations();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[pool.mkfs_operation_count..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("发布有一条根槽写");
    RecordedFlow {
        name,
        base: pool.memory_pool_after_mkfs(),
        writes,
        segments,
        judged_root_index,
        versions: vec![PublishedVersion {
            instance: InstanceGeneration(PUBLISH_ROOT_IDENTITY.0),
            checkpoint_txg: CheckpointTxg(PUBLISH_ROOT_IDENTITY.1),
            content: file_content(),
        }],
    }
}

/// 被判的根（`judged_root_index` 那次根槽写）的 (实例代号, checkpoint_txg)。
fn judged_root_identity(flow: &RecordedFlow) -> (u32, u64) {
    root_identity_of_the_root_record(
        flow.writes[flow.judged_root_index]
            .bytes()
            .expect("根槽写带字节"),
    )
}

/// 一次根槽写写下的根记录的 (实例代号, checkpoint_txg)。
fn root_identity_of_the_root_record(bytes: &[u8]) -> (u32, u64) {
    (
        u32::from_le_bytes(
            bytes[ROOT_RECORD_INSTANCE_OFFSET..ROOT_RECORD_INSTANCE_OFFSET + 4]
                .try_into()
                .expect("4"),
        ),
        u64::from_le_bytes(
            bytes[ROOT_RECORD_CHECKPOINT_TXG_OFFSET..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
                .try_into()
                .expect("8"),
        ),
    )
}

/// 根记录重封整槽校验和（罩整个槽宽、字段按 0 参与）。
fn reseal_root_record(bytes: &mut [u8]) {
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(
        bytes,
        bytes.len(),
        ROOT_RECORD_CHECKSUM_FIELD_OFFSET,
    );
    bytes[ROOT_RECORD_CHECKSUM_FIELD_OFFSET
        ..ROOT_RECORD_CHECKSUM_FIELD_OFFSET + ROOT_RECORD_CHECKSUM_BYTES]
        .copy_from_slice(&digest);
}

/// 在流尾另起一段，放这几次写（它们在最后全部持久的那个状态里都落了）。
fn append_a_segment_of_writes(flow: &mut RecordedFlow, writes: Vec<RetainedWrite>) {
    let first = flow.writes.len();
    let count = writes.len();
    flow.writes.extend(writes);
    flow.segments.push((first..first + count).collect());
}

/// 流尾另起一段写的那条根带不带卸载记号（根记录 flags 位 0）。
#[derive(Clone, Copy, Debug)]
enum UnmountMarkerOfTheAppendedRoot {
    Written,
    NotWritten,
}

/// 流尾另起一段写一条根：抄被判那条根（`judged_root_index` 那次根槽写）的字节，checkpoint_txg、回退下界 F、flags 位 0 照给的改，
/// 实例代号与各条指针照抄，重封整槽校验和，写到这个 txg 按发布规则算出的那个根槽上（`singlefs_core::root_ring::target_for_publish`，
/// I-7.17 不因为它红）；那个槽里原来的根被它盖掉。版本表多一版：实例与内容同被判那条根那一版。
fn append_a_copy_of_the_judged_root_at_a_later_txg(
    flow: &mut RecordedFlow,
    checkpoint_txg: u64,
    rollback_floor: u64,
    unmount_marker: UnmountMarkerOfTheAppendedRoot,
) {
    let source = flow.writes[flow.judged_root_index].clone();
    assert_eq!(source.kind, StepKind::RootRecordFua, "抄的是根槽写");
    let WrittenContents::Bytes(mut bytes) = source.contents else {
        panic!("根槽写是普通写");
    };
    bytes[ROOT_RECORD_CHECKPOINT_TXG_OFFSET..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
        .copy_from_slice(&checkpoint_txg.to_le_bytes());
    bytes[ROOT_RECORD_ROLLBACK_FLOOR_OFFSET..ROOT_RECORD_ROLLBACK_FLOOR_OFFSET + 8]
        .copy_from_slice(&rollback_floor.to_le_bytes());
    let flags: u32 = match unmount_marker {
        UnmountMarkerOfTheAppendedRoot::Written => 1,
        UnmountMarkerOfTheAppendedRoot::NotWritten => 0,
    };
    bytes[ROOT_RECORD_FLAGS_OFFSET..ROOT_RECORD_FLAGS_OFFSET + 4]
        .copy_from_slice(&flags.to_le_bytes());
    reseal_root_record(&mut bytes);
    let target = singlefs_core::root_ring::target_for_publish(
        CheckpointTxg(checkpoint_txg),
        geometry().root_ring_slots_per_region,
    );
    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let offset =
        singlefs_core::root_ring::slot_offset(target, geometry().fixed_structure_slot_spacing);
    append_a_segment_of_writes(
        flow,
        vec![RetainedWrite {
            device,
            kind: StepKind::RootRecordFua,
            is_force_unit_access: source.is_force_unit_access,
            offset,
            contents: WrittenContents::Bytes(bytes),
        }],
    );
    let (judged_instance, judged_txg) = judged_root_identity(flow);
    let content = flow
        .versions
        .iter()
        .find(|version| {
            (version.instance.0, version.checkpoint_txg.0) == (judged_instance, judged_txg)
        })
        .unwrap_or_else(|| {
            panic!(
                "流 {} 的版本表里有被判那条根那一版（实例 {judged_instance}、txg {judged_txg}）",
                flow.name
            )
        })
        .content
        .clone();
    flow.versions.push(PublishedVersion {
        instance: InstanceGeneration(judged_instance),
        checkpoint_txg: CheckpointTxg(checkpoint_txg),
        content,
    });
}

/// 新池新建文件那条流的流尾逐段写三条抄发布那条根的根（文件头那张根环），txg 27 那条把 F 从 2 抬到 `raised_floor`。
fn record_new_pool_with_a_floor_raise_over_the_slot_of_the_publish(
    tag: &str,
    name: &'static str,
    raised_floor: u64,
) -> RecordedFlow {
    let mut flow = record_new_pool_file_creation(tag, name);
    assert_eq!(
        judged_root_identity(&flow),
        PUBLISH_ROOT_IDENTITY,
        "被判的根是新建文件那次发布"
    );
    assert_eq!(
        singlefs_format::ROOT_RING_REGIONS * geometry().root_ring_slots_per_region.count(),
        ROOT_RING_SLOT_COUNT_OF_THE_NEW_POOL,
        "根环 3S = 24：txg 25、26、27 依次落在 txg 1、2、3 那三槽上"
    );
    for (checkpoint_txg, rollback_floor, unmount_marker) in [
        (
            TXG_OF_THE_UNMOUNT_MARKED_ROOT,
            FLOOR_BEFORE_THE_RAISE,
            UnmountMarkerOfTheAppendedRoot::Written,
        ),
        (
            TXG_OF_THE_ROOT_BEFORE_THE_RAISE,
            FLOOR_BEFORE_THE_RAISE,
            UnmountMarkerOfTheAppendedRoot::NotWritten,
        ),
        (
            TXG_OF_THE_ROOT_RAISING_THE_FLOOR,
            raised_floor,
            UnmountMarkerOfTheAppendedRoot::NotWritten,
        ),
    ] {
        append_a_copy_of_the_judged_root_at_a_later_txg(
            &mut flow,
            checkpoint_txg,
            rollback_floor,
            unmount_marker,
        );
    }
    flow
}

/// 双实例那条流覆盖写那一版的内容（实例 1、txg 4；卸载那一串与实例 2 挂载那几次发布都带着它）。
fn content_of_the_overwrite_before_the_unmount() -> Vec<u8> {
    (0..FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 流里有没有一份 `instance` 实例、checkpoint_txg 是 `checkpoint_txg`、带提交标记的 journal 记录写。
fn has_a_committed_journal_record_of(
    flow: &RecordedFlow,
    instance: u32,
    checkpoint_txg: u64,
) -> bool {
    flow.writes.iter().any(|write| {
        write.kind == StepKind::JournalRecord
            && write.bytes().is_some_and(|bytes| {
                journal_record_instance_and_counter(bytes).0 == instance
                    && u64::from_le_bytes(
                        bytes[JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET
                            ..JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET + 8]
                            .try_into()
                            .expect("8"),
                    ) == checkpoint_txg
                    && bytes[JOURNAL_RECORD_COMMIT_MARKER_OFFSET]
                        == JOURNAL_RECORD_COMMIT_MARKER_PRESENT
            })
    })
}

/// 双实例那条流（文件头「跨实例那一支」）：新池新建文件（实例 1、txg 3）之后覆盖写一次（txg 4），正常卸载（F 抬到 4、三次带卸载记号的空发布，
/// 录制流上记成卸载入口发的，C557（卸载记号只由卸载入口打没有检查）），关掉镜像重开可写挂载（实例 2）。被判的根是流里最后一条根槽写（实例 2 暖机那条）。
/// 前提逐条钉：根槽写依次的身份、卸载抬到的 F 与它那几次空发布、实例 1 那两版（中间那一版与最后一版）在 journal 里有带提交标记的记录。
fn record_new_pool_overwritten_then_unmounted_and_mounted_writable_again(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let content_of_the_overwrite = content_of_the_overwrite_before_the_unmount();
    pool.output = {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &pool.output,
            FirstFile {
                content: &content_of_the_overwrite,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            InstanceGeneration(UNMOUNTED_INSTANCE),
        )
        .expect("覆盖写")
    };
    let stream = pool.stream.clone();
    let unmounted = {
        let mut current = PoolVersion::WithFile(Arc::new(pool.output.clone()));
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let allocator = &mut pool.allocator;
        stream
            .record_entry(RecordedPublishEntry::Unmount, || {
                unmount(
                    &parameters(),
                    devices,
                    allocator,
                    &mut current,
                    ShadowLedger::On,
                )
            })
            .expect("正常卸载")
    };
    let floor_raise_of_the_unmount = match unmounted {
        Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised,
        Unmounted::NothingWrittenOnAVersionWithoutFile { current } => {
            panic!("流 {name}：带文件的一版上卸载报成「树表 0 条、一个字节都不写」：{current:?}")
        }
    };
    assert_eq!(
        floor_raise_of_the_unmount.rollback_floor,
        CheckpointTxg(FLOOR_RAISED_BY_THE_UNMOUNT),
        "流 {name}：卸载把 F 抬到覆盖写那一版的 txg"
    );
    assert_eq!(
        floor_raise_of_the_unmount
            .publishes
            .iter()
            .map(|publish| (publish.root.instance.0, publish.root.checkpoint_txg.0))
            .collect::<Vec<_>>(),
        ROOT_IDENTITIES_OF_THE_FLOW_THROUGH_THE_UNMOUNT_AND_THE_MOUNT_AFTER_IT
            .iter()
            .copied()
            .filter(|(instance, checkpoint_txg)| *instance == UNMOUNTED_INSTANCE
                && *checkpoint_txg > FLOOR_RAISED_BY_THE_UNMOUNT)
            .collect::<Vec<_>>(),
        "流 {name}：卸载推三次带卸载记号的空发布（现行 txg 4 除以 3 余 1）"
    );
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("卸载之后重开可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("卸载之后重开，现行那一版带文件");
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let entry_spans: Vec<RecordedEntrySpan> = pool
        .stream
        .entry_spans()
        .into_iter()
        .map(|span| RecordedEntrySpan {
            entry: span.entry,
            operations: span.operations.start - pool.mkfs_operation_count
                ..span.operations.end - pool.mkfs_operation_count,
        })
        .collect();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &operations[pool.mkfs_operation_count..],
        &entry_spans,
        &geometry(),
    );
    let root_identities: Vec<(u32, u64)> = writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .map(|write| root_identity_of_the_root_record(write.bytes().expect("根槽写带字节")))
        .collect();
    assert_eq!(
        root_identities,
        ROOT_IDENTITIES_OF_THE_FLOW_THROUGH_THE_UNMOUNT_AND_THE_MOUNT_AFTER_IT.to_vec(),
        "流 {name} 的根槽写依次是暖机两代、新建文件、覆盖写、卸载那一串、实例 2 挂载那几次（推的；对不上就照打出的身份改流尾那几条根的 txg 与 F）"
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("挂载写了根槽");
    let versions: Vec<PublishedVersion> = root_identities
        .iter()
        .filter(|(_, checkpoint_txg)| *checkpoint_txg >= PUBLISH_ROOT_IDENTITY.1)
        .map(|(instance, checkpoint_txg)| PublishedVersion {
            instance: InstanceGeneration(*instance),
            checkpoint_txg: CheckpointTxg(*checkpoint_txg),
            content: if (*instance, *checkpoint_txg) == PUBLISH_ROOT_IDENTITY {
                file_content()
            } else {
                content_of_the_overwrite.clone()
            },
        })
        .collect();
    let flow = RecordedFlow {
        name,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
    };
    for checkpoint_txg in [
        INTERMEDIATE_TXG_OF_THE_UNMOUNTED_INSTANCE,
        LAST_TXG_OF_THE_UNMOUNTED_INSTANCE,
    ] {
        assert!(
            has_a_committed_journal_record_of(&flow, UNMOUNTED_INSTANCE, checkpoint_txg),
            "流 {name}：实例 {UNMOUNTED_INSTANCE} txg {checkpoint_txg} 那次发布在 journal 里有带提交标记的记录（重建抬 F 那一刻的环按它认那一槽）"
        );
    }
    flow
}

/// 双实例那条流的流尾逐段抄实例 2 最后那条根：`txgs_before_the_raise` 那几条带卸载抬到的 F（不抬），`txg_of_the_raise` 那条把 F 抬到 `raised_floor`、
/// 盖掉落点式在它之下最大那一版（txg_of_the_raise − 3S）的那一槽；那一版是实例 1 的，跨实例那一支要看实例 2 的实例表里实例 1 那一行。
fn record_a_floor_raise_over_the_slot_of_a_version_of_the_unmounted_instance(
    tag: &str,
    name: &'static str,
    txgs_before_the_raise: &[u64],
    txg_of_the_raise: u64,
    raised_floor: u64,
) -> RecordedFlow {
    let mut flow = record_new_pool_overwritten_then_unmounted_and_mounted_writable_again(tag, name);
    assert_eq!(
        judged_root_identity(&flow).0,
        INSTANCE_MOUNTED_AFTER_THE_UNMOUNT,
        "流 {name}：被判的根（流尾抄的就是它）是卸载之后重开挂载的那个实例的"
    );
    assert_eq!(
        singlefs_format::ROOT_RING_REGIONS * geometry().root_ring_slots_per_region.count(),
        ROOT_RING_SLOT_COUNT_OF_THE_NEW_POOL,
        "根环 3S = 24：流尾 txg t 那条根盖掉的是 txg t − 24 那一槽"
    );
    for checkpoint_txg in txgs_before_the_raise {
        append_a_copy_of_the_judged_root_at_a_later_txg(
            &mut flow,
            *checkpoint_txg,
            FLOOR_RAISED_BY_THE_UNMOUNT,
            UnmountMarkerOfTheAppendedRoot::NotWritten,
        );
    }
    append_a_copy_of_the_judged_root_at_a_later_txg(
        &mut flow,
        txg_of_the_raise,
        raised_floor,
        UnmountMarkerOfTheAppendedRoot::NotWritten,
    );
    flow
}

/// 下一条记录该带的反向链：这一条 496 字节头、头校验和字段按 0 参与的 CRC-32C。
fn journal_chain_value(bytes: &[u8]) -> u32 {
    let mut header = bytes[..JOURNAL_RECORD_HEADER_BYTES].to_vec();
    header[JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET
        ..JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET + JOURNAL_RECORD_HEADER_CHECKSUM_BYTES]
        .fill(0);
    singlefs_core::checksum::crc32_castagnoli(&header)
}

/// journal 记录重封：载荷校验和（罩点名项）再整条头校验和（罩 4096 字节）。
fn reseal_journal_record(bytes: &mut [u8]) {
    let named_entries = usize::try_from(u32::from_le_bytes(
        bytes[JOURNAL_RECORD_NAMED_ENTRY_COUNT_OFFSET..JOURNAL_RECORD_NAMED_ENTRY_COUNT_OFFSET + 4]
            .try_into()
            .expect("4"),
    ))
    .expect("点名项数");
    let payload_checksum = singlefs_core::checksum::crc32_castagnoli(
        &bytes[JOURNAL_RECORD_HEADER_BYTES
            ..JOURNAL_RECORD_HEADER_BYTES + named_entries * JOURNAL_RECORD_NAMED_ENTRY_BYTES],
    );
    bytes[JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET..JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET + 4]
        .copy_from_slice(&payload_checksum.to_le_bytes());
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(
        bytes,
        JOURNAL_RECORD_CHECKSUMMED_BYTES,
        JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET,
    );
    bytes[JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET
        ..JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET + JOURNAL_RECORD_HEADER_CHECKSUM_BYTES]
        .copy_from_slice(&digest);
}

/// 一条 journal 记录写的 (实例代号, 计数器)。
fn journal_record_instance_and_counter(bytes: &[u8]) -> (u32, u64) {
    let mut counter = [0u8; 8];
    counter[..JOURNAL_RECORD_COUNTER_BYTES].copy_from_slice(
        &bytes[JOURNAL_RECORD_COUNTER_OFFSET
            ..JOURNAL_RECORD_COUNTER_OFFSET + JOURNAL_RECORD_COUNTER_BYTES],
    );
    (
        u32::from_le_bytes(
            bytes[JOURNAL_RECORD_INSTANCE_OFFSET..JOURNAL_RECORD_INSTANCE_OFFSET + 4]
                .try_into()
                .expect("4"),
        ),
        u64::from_le_bytes(counter),
    )
}

/// 重封过的记录之后，同盘、同实例、计数器大 1 的那一条的反向链换成它的新链值、重封，一路往后传。
fn carry_the_back_chain_forward_from(
    flow: &mut RecordedFlow,
    mut journal_records_resealed: Vec<usize>,
) {
    while let Some(record_index) = journal_records_resealed.pop() {
        let (record_device, (instance, counter), chain) = {
            let write = &flow.writes[record_index];
            let bytes = write.bytes().expect("记录写带字节");
            (
                write.device,
                journal_record_instance_and_counter(bytes),
                journal_chain_value(bytes),
            )
        };
        for next_index in 0..flow.writes.len() {
            if flow.writes[next_index].kind != StepKind::JournalRecord
                || flow.writes[next_index].device != record_device
            {
                continue;
            }
            let WrittenContents::Bytes(bytes) = &mut flow.writes[next_index].contents else {
                continue;
            };
            if journal_record_instance_and_counter(bytes) != (instance, counter + 1)
                || bytes[JOURNAL_RECORD_BACK_CHAIN_OFFSET..JOURNAL_RECORD_BACK_CHAIN_OFFSET + 4]
                    == chain.to_le_bytes()
            {
                continue;
            }
            bytes[JOURNAL_RECORD_BACK_CHAIN_OFFSET..JOURNAL_RECORD_BACK_CHAIN_OFFSET + 4]
                .copy_from_slice(&chain.to_le_bytes());
            reseal_journal_record(bytes);
            journal_records_resealed.push(next_index);
        }
    }
}

/// 新建文件那次发布（实例 1、txg 3）的每一份 journal 记录提交标记写成 0、重封、同实例后面的记录重链：
/// journal 里不再有 txg 3 带提交标记的那一版，抬 F 那一刻 txg 3 那一槽认不出。
fn clear_the_commit_markers_of_the_publish_records(flow: &mut RecordedFlow) {
    let (instance, checkpoint_txg) = PUBLISH_ROOT_IDENTITY;
    let records: Vec<usize> = flow
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::JournalRecord
                && write.bytes().is_some_and(|bytes| {
                    journal_record_instance_and_counter(bytes).0 == instance
                        && u64::from_le_bytes(
                            bytes[JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET
                                ..JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET + 8]
                                .try_into()
                                .expect("8"),
                        ) == checkpoint_txg
                })
        })
        .map(|(index, _)| index)
        .collect();
    assert!(
        !records.is_empty(),
        "流 {} 里有新建文件那次发布（实例 {instance}、txg {checkpoint_txg}）的 journal 记录",
        flow.name
    );
    for index in &records {
        let WrittenContents::Bytes(bytes) = &mut flow.writes[*index].contents else {
            panic!("记录写是普通写");
        };
        assert_eq!(
            bytes[JOURNAL_RECORD_COMMIT_MARKER_OFFSET], JOURNAL_RECORD_COMMIT_MARKER_PRESENT,
            "流 {}：改之前新建文件那次发布的记录带提交标记",
            flow.name
        );
        bytes[JOURNAL_RECORD_COMMIT_MARKER_OFFSET] = JOURNAL_RECORD_COMMIT_MARKER_ABSENT;
        reseal_journal_record(bytes);
    }
    carry_the_back_chain_forward_from(flow, records);
}

/// CPU 池级 checker 在一个状态上给 I-7.9 判成的那一种（只记对拍之外要钉的那几样）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum RollbackFloorVerdictOfAState {
    Holds,
    Violated(String),
    PartiallyJudged(String),
    NotApplicable,
    NotReported,
}

/// 一条流逐状态比完的结果：CPU 每个状态 I-7.9 判成什么，GPU 与 CPU 对拍字不同的状态（前几个带说明）。
struct ComparisonOfTheFlow {
    rollback_floor_verdicts_of_the_cpu: Vec<RollbackFloorVerdictOfAState>,
    mismatches: Vec<String>,
}

/// 一条流逐状态在 GPU 与 CPU 上各判一遍、比对拍字；没卡、借不到卡或 CPU 判不完时 None（报「本次未跑」）。
fn compare_the_flow_on_every_state(
    test_name: &str,
    flow: &RecordedFlow,
) -> Option<ComparisonOfTheFlow> {
    compare_the_flow_expanding_the_segments_from(test_name, flow, 0)
}

/// 同 [`compare_the_flow_on_every_state`]，只展开下标不小于 `first_expanded_segment` 的段（更早的段只以整段持久进入后面的状态）：
/// 流前头几十次发布的段逐个真子集展开时状态数上万，而这几条用例判的是流尾抬 F 那条根，抬 F 那一刻的样子只在流尾那几段落了之后的状态上。
fn compare_the_flow_expanding_the_segments_from(
    test_name: &str,
    flow: &RecordedFlow,
    first_expanded_segment: usize,
) -> Option<ComparisonOfTheFlow> {
    let cards = gpu_card_pool_or_report_not_run(test_name)?;
    let expansion = |segment_index: usize, segment: &[usize]| {
        if segment_index >= first_expanded_segment && segment.len() <= EXPAND_UP_TO {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let tables = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    })
    .expect("建得出表");
    let state_count =
        crash_point_replay_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
    assert_eq!(tables.state_count, state_count);
    let gpu: Vec<[u32; VERDICT_WORDS]> =
        judge_every_state_on_a_card_leased_from_the_pool(cards, test_name, &tables)?;
    let mut cpu: Vec<CrossCheckWords> = Vec::new();
    let mut rollback_floor_verdicts_of_the_cpu: Vec<RollbackFloorVerdictOfAState> = Vec::new();
    let judged_on_the_cpu = sample_crash_states_on_the_cpu(
        CrashPointReplayItem {
            name: test_name,
            setting_file: THIS_FILE,
        },
        &flow.base,
        &flow.writes,
        &flow.segments,
        flow.judged_root_index,
        &flow.versions,
        &expansion,
        0..state_count,
        &mut |_ordinal, _image, judgement| {
            cpu.push(cross_check_words_of_cpu_judgement(judgement));
            rollback_floor_verdicts_of_the_cpu.push(
                match judgement
                    .pool_checker
                    .iter()
                    .find(|(invariant, _)| *invariant == "I-7.9")
                    .map(|(_, verdict)| verdict)
                {
                    None => RollbackFloorVerdictOfAState::NotReported,
                    Some(InvariantVerdict::Holds) => RollbackFloorVerdictOfAState::Holds,
                    Some(InvariantVerdict::Violated(detail)) => {
                        RollbackFloorVerdictOfAState::Violated(detail.clone())
                    }
                    Some(InvariantVerdict::PartiallyJudged(unjudged)) => {
                        RollbackFloorVerdictOfAState::PartiallyJudged(unjudged.clone())
                    }
                    Some(InvariantVerdict::NotApplicable(_)) => {
                        RollbackFloorVerdictOfAState::NotApplicable
                    }
                },
            );
        },
    )
    .judged()?;
    assert_eq!(judged_on_the_cpu, state_count);
    assert_eq!(cpu.len(), gpu.len(), "两边判的状态数相同");
    let mismatches: Vec<String> = cpu
        .iter()
        .zip(&gpu)
        .enumerate()
        .filter_map(|(ordinal, (cpu_words, gpu_verdict))| {
            let gpu_words = cross_check_words_of_gpu_verdict(gpu_verdict);
            (!cross_check_words_agree_in_the_cross_check(&gpu_words, cpu_words))
                .then(|| format!("状态 {ordinal}：CPU {cpu_words:?}；GPU {gpu_words:?}"))
        })
        .collect();
    Some(ComparisonOfTheFlow {
        rollback_floor_verdicts_of_the_cpu,
        mismatches,
    })
}

/// GPU 与 CPU 逐状态的对拍字相同。
fn assert_the_gpu_agrees_with_the_cpu_on_every_state(
    flow: &RecordedFlow,
    comparison: &ComparisonOfTheFlow,
) {
    assert!(
        comparison.mismatches.is_empty(),
        "流 {}：{} 个状态 GPU 与 CPU 对拍的字不同（前几个）：\n{}",
        flow.name,
        comparison.mismatches.len(),
        comparison
            .mismatches
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 合法写者那张：F 抬到重建的环的上限 3。CPU 至少一个状态上判过 I-7.9 而成立、一个状态都不红它，GPU 逐状态相同。
#[test]
fn the_gpu_judge_holds_a_floor_raised_to_the_ceiling_of_the_ring_rebuilt_from_the_journal_like_the_cpu(
) {
    const TEST_NAME: &str = "the_gpu_judge_holds_a_floor_raised_to_the_ceiling_of_the_ring_rebuilt_from_the_journal_like_the_cpu";
    let flow = record_new_pool_with_a_floor_raise_over_the_slot_of_the_publish(
        "crash-judge-gpu-c717-legal-writer",
        "new-pool-floor-raise-to-the-ceiling-of-the-ring-rebuilt-from-the-journal",
        FLOOR_RAISED_TO_THE_CEILING_OF_THE_RECONSTRUCTED_RING,
    );
    let Some(comparison) = compare_the_flow_on_every_state(TEST_NAME, &flow) else {
        return;
    };
    let verdicts = &comparison.rollback_floor_verdicts_of_the_cpu;
    assert!(
        verdicts.contains(&RollbackFloorVerdictOfAState::Holds)
            && !verdicts
                .iter()
                .any(|verdict| matches!(verdict, RollbackFloorVerdictOfAState::Violated(_))),
        "流 {} 上 CPU 判过 I-7.9 而成立、一个状态都不红它（逐状态：{verdicts:?}）",
        flow.name
    );
    assert_the_gpu_agrees_with_the_cpu_on_every_state(&flow, &comparison);
}

/// 坏写者那张：F 抬到 4，窗口里的上限 25 判成立，读回全零的槽能住的最大 txg 4 不低于它，按 journal 重建的抬 F 那一刻的环上限 3，红。CPU 至少一个状态上按重建的环红 I-7.9，
/// GPU 逐状态相同。GPU 不读 journal、退回只看窗口时那几个状态判成立，对不上。
#[test]
fn the_gpu_judge_reddens_a_floor_raised_past_the_ceiling_of_the_ring_rebuilt_from_the_journal_like_the_cpu(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_a_floor_raised_past_the_ceiling_of_the_ring_rebuilt_from_the_journal_like_the_cpu";
    let flow = record_new_pool_with_a_floor_raise_over_the_slot_of_the_publish(
        "crash-judge-gpu-c717-bad-writer",
        "new-pool-floor-raise-past-the-ceiling-of-the-ring-rebuilt-from-the-journal",
        FLOOR_RAISED_PAST_THE_CEILING_OF_THE_RECONSTRUCTED_RING,
    );
    let Some(comparison) = compare_the_flow_on_every_state(TEST_NAME, &flow) else {
        return;
    };
    let verdicts = &comparison.rollback_floor_verdicts_of_the_cpu;
    assert!(
        verdicts.iter().any(|verdict| matches!(
            verdict,
            RollbackFloorVerdictOfAState::Violated(detail) if detail.contains("按 journal 重建")
        )),
        "流 {} 上 CPU 至少一个状态按 journal 重建的环红 I-7.9（逐状态：{verdicts:?}）",
        flow.name
    );
    assert_the_gpu_agrees_with_the_cpu_on_every_state(&flow, &comparison);
}

/// 坏写者那张、journal 罩不到：txg 3 那次发布的记录不带提交标记，抬 F 那一刻 txg 3 那一槽认不出，环重建不出。
/// CPU 至少一个状态上 I-7.9 报射程里判不了、一个状态都不红它，GPU 逐状态相同（那一条判到位为 0）。
#[test]
fn the_gpu_judge_leaves_the_floor_raise_partially_judged_when_the_journal_no_longer_names_the_overwritten_version_like_the_cpu(
) {
    const TEST_NAME: &str = "the_gpu_judge_leaves_the_floor_raise_partially_judged_when_the_journal_no_longer_names_the_overwritten_version_like_the_cpu";
    let mut flow = record_new_pool_with_a_floor_raise_over_the_slot_of_the_publish(
        "crash-judge-gpu-c717-journal-cleared",
        "new-pool-floor-raise-whose-overwritten-version-the-journal-no-longer-names",
        FLOOR_RAISED_PAST_THE_CEILING_OF_THE_RECONSTRUCTED_RING,
    );
    clear_the_commit_markers_of_the_publish_records(&mut flow);
    let Some(comparison) = compare_the_flow_on_every_state(TEST_NAME, &flow) else {
        return;
    };
    let verdicts = &comparison.rollback_floor_verdicts_of_the_cpu;
    assert!(
        verdicts.iter().any(|verdict| matches!(
            verdict,
            RollbackFloorVerdictOfAState::PartiallyJudged(unjudged) if unjudged.contains("journal 罩不到")
        )) && !verdicts
            .iter()
            .any(|verdict| matches!(verdict, RollbackFloorVerdictOfAState::Violated(_))),
        "流 {} 上 CPU 至少一个状态 I-7.9 报 journal 罩不到、一个状态都不红它（逐状态：{verdicts:?}）",
        flow.name
    );
    assert_the_gpu_agrees_with_the_cpu_on_every_state(&flow, &comparison);
}

/// 跨实例、合法写者那张（C935（I-7.9 按 journal 重建时 u 属于另一个实例那一支没有用例））：实例 2 的 txg 29 把 F 从 4 抬到 5，
/// 抬 F 那一刻 txg 5 那一槽住的是实例 1 中间那一版（5 < 实例 1 那一行的 T 7，认作根槽落了盘），重建的环上限 5，成立。
/// CPU 至少一个状态上判过 I-7.9 而成立、一个状态都不红它、一个状态都不报射程里判不了（认不出 txg 5 时报的是射程里判不了），GPU 逐状态相同。
#[test]
fn the_gpu_judge_holds_a_floor_raised_to_the_ceiling_of_the_ring_holding_a_version_of_the_unmounted_instance_like_the_cpu(
) {
    const TEST_NAME: &str = "the_gpu_judge_holds_a_floor_raised_to_the_ceiling_of_the_ring_holding_a_version_of_the_unmounted_instance_like_the_cpu";
    assert_eq!(
        TXG_OF_THE_RAISE_OVER_THE_INTERMEDIATE_VERSION - ROOT_RING_SLOT_COUNT_OF_THE_NEW_POOL,
        INTERMEDIATE_TXG_OF_THE_UNMOUNTED_INSTANCE,
        "抬 F 那条根盖掉的是实例 1 中间那一版那一槽"
    );
    let flow = record_a_floor_raise_over_the_slot_of_a_version_of_the_unmounted_instance(
        "crash-judge-gpu-c935-legal-writer",
        "remounted-pool-floor-raise-to-the-ceiling-of-the-ring-holding-a-version-of-the-unmounted-instance",
        &TXGS_BEFORE_THE_RAISE_OVER_THE_INTERMEDIATE_VERSION,
        TXG_OF_THE_RAISE_OVER_THE_INTERMEDIATE_VERSION,
        FLOOR_RAISED_TO_THE_CEILING_OF_THE_RING_HOLDING_THE_INTERMEDIATE_VERSION,
    );
    let Some(comparison) = compare_the_flow_on_every_state(TEST_NAME, &flow) else {
        return;
    };
    let verdicts = &comparison.rollback_floor_verdicts_of_the_cpu;
    assert!(
        verdicts.contains(&RollbackFloorVerdictOfAState::Holds)
            && !verdicts.iter().any(|verdict| matches!(
                verdict,
                RollbackFloorVerdictOfAState::Violated(_)
                    | RollbackFloorVerdictOfAState::PartiallyJudged(_)
            )),
        "流 {} 上 CPU 判过 I-7.9 而成立、一个状态都不红它、一个状态都不报射程里判不了（逐状态：{verdicts:?}）",
        flow.name
    );
    assert_the_gpu_agrees_with_the_cpu_on_every_state(&flow, &comparison);
}

/// 跨实例、坏写者那张（C935）：实例 2 的 txg 29 把 F 从 4 抬到 6，窗口（txg > 5）里的上限 6 判成立而定不住，按 journal 重建的环
/// 认出实例 1 中间那一版 txg 5、上限 5，红。CPU 至少一个状态上按重建的环红 I-7.9、最旧有效根是 txg 5，GPU 逐状态相同。
#[test]
fn the_gpu_judge_reddens_a_floor_raised_past_the_ceiling_of_the_ring_holding_a_version_of_the_unmounted_instance_like_the_cpu(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_a_floor_raised_past_the_ceiling_of_the_ring_holding_a_version_of_the_unmounted_instance_like_the_cpu";
    let flow = record_a_floor_raise_over_the_slot_of_a_version_of_the_unmounted_instance(
        "crash-judge-gpu-c935-bad-writer",
        "remounted-pool-floor-raise-past-the-ceiling-of-the-ring-holding-a-version-of-the-unmounted-instance",
        &TXGS_BEFORE_THE_RAISE_OVER_THE_INTERMEDIATE_VERSION,
        TXG_OF_THE_RAISE_OVER_THE_INTERMEDIATE_VERSION,
        FLOOR_RAISED_PAST_THE_CEILING_OF_THE_RING_HOLDING_THE_INTERMEDIATE_VERSION,
    );
    let Some(comparison) = compare_the_flow_on_every_state(TEST_NAME, &flow) else {
        return;
    };
    let verdicts = &comparison.rollback_floor_verdicts_of_the_cpu;
    let oldest_valid_root_of_the_reconstructed_ring =
        format!("最旧有效根 txg {INTERMEDIATE_TXG_OF_THE_UNMOUNTED_INSTANCE}");
    assert!(
        verdicts.iter().any(|verdict| matches!(
            verdict,
            RollbackFloorVerdictOfAState::Violated(detail)
                if detail.contains("按 journal 重建")
                    && detail.contains(&oldest_valid_root_of_the_reconstructed_ring)
        )),
        "流 {} 上 CPU 至少一个状态按 journal 重建的环红 I-7.9、那个环的{oldest_valid_root_of_the_reconstructed_ring}（逐状态：{verdicts:?}）",
        flow.name
    );
    assert_the_gpu_agrees_with_the_cpu_on_every_state(&flow, &comparison);
}

/// 跨实例、被盖的是实例 1 最后一版、两种可能的环都成立的那张（C935；u = T 那一形）：实例 2 的 txg 31 把 F 从 4 抬到 6，抬 F 那一刻 txg 7 那一槽住的
/// 可能是实例 1 的最后一版（u = 实例 1 那一行的 T：它的根槽可以没落），也可能空着；住 txg 7 那一种上限 7、空着那一种上限 8，F 6 两种都不越过，判成立。
/// CPU 至少一个状态上判过 I-7.9 而成立、一个状态都不红它、一个状态都不报射程里判不了，GPU 逐状态相同。u ≥ T 退回报认不出时那一边报射程里判不了，对不上。
#[test]
fn the_gpu_judge_holds_a_floor_raise_over_the_slot_of_the_last_version_of_the_unmounted_instance_within_both_possible_rings_like_the_cpu(
) {
    const TEST_NAME: &str = "the_gpu_judge_holds_a_floor_raise_over_the_slot_of_the_last_version_of_the_unmounted_instance_within_both_possible_rings_like_the_cpu";
    assert_eq!(
        TXG_OF_THE_RAISE_OVER_THE_LAST_VERSION - ROOT_RING_SLOT_COUNT_OF_THE_NEW_POOL,
        LAST_TXG_OF_THE_UNMOUNTED_INSTANCE,
        "抬 F 那条根盖掉的是实例 1 最后一版那一槽"
    );
    let flow = record_a_floor_raise_over_the_slot_of_a_version_of_the_unmounted_instance(
        "crash-judge-gpu-c935-last-version-within-both",
        "remounted-pool-floor-raise-over-the-slot-of-the-last-version-within-both-possible-rings",
        &TXGS_BEFORE_THE_RAISE_OVER_THE_LAST_VERSION,
        TXG_OF_THE_RAISE_OVER_THE_LAST_VERSION,
        FLOOR_RAISED_WITHIN_BOTH_POSSIBLE_RINGS_OVER_THE_SLOT_OF_THE_LAST_VERSION,
    );
    let Some(comparison) = compare_the_flow_on_every_state(TEST_NAME, &flow) else {
        return;
    };
    let verdicts = &comparison.rollback_floor_verdicts_of_the_cpu;
    assert!(
        verdicts.contains(&RollbackFloorVerdictOfAState::Holds)
            && !verdicts.iter().any(|verdict| matches!(
                verdict,
                RollbackFloorVerdictOfAState::Violated(_)
                    | RollbackFloorVerdictOfAState::PartiallyJudged(_)
            )),
        "流 {} 上 CPU 判过 I-7.9 而成立、一个状态都不红它、一个状态都不报射程里判不了（逐状态：{verdicts:?}）",
        flow.name
    );
    assert_the_gpu_agrees_with_the_cpu_on_every_state(&flow, &comparison);
}

/// 跨实例、被盖的是实例 1 最后一版、两种可能的环判得不一致的那张（C935；u = T 那一形）：实例 2 的 txg 31 把 F 从 4 抬到 8，住 txg 7 那一种上限 7、红，
/// 空着那一种上限 8、成立，判得不一致，射程里判不了。CPU 至少一个状态这样报、一个状态都不红它，GPU 逐状态相同（那一条判到位为 0）。
/// 跨实例那一支不看实例表的行 T（当成同一实例）时那一边只认 txg 7、按上限 7 判红，对不上。
#[test]
fn the_gpu_judge_leaves_a_floor_raise_over_the_slot_of_the_last_version_of_the_unmounted_instance_partially_judged_like_the_cpu(
) {
    const TEST_NAME: &str = "the_gpu_judge_leaves_a_floor_raise_over_the_slot_of_the_last_version_of_the_unmounted_instance_partially_judged_like_the_cpu";
    assert_eq!(
        TXG_OF_THE_RAISE_OVER_THE_LAST_VERSION - ROOT_RING_SLOT_COUNT_OF_THE_NEW_POOL,
        LAST_TXG_OF_THE_UNMOUNTED_INSTANCE,
        "抬 F 那条根盖掉的是实例 1 最后一版那一槽"
    );
    let flow = record_a_floor_raise_over_the_slot_of_a_version_of_the_unmounted_instance(
        "crash-judge-gpu-c935-last-version",
        "remounted-pool-floor-raise-over-the-slot-of-the-last-version-of-the-unmounted-instance",
        &TXGS_BEFORE_THE_RAISE_OVER_THE_LAST_VERSION,
        TXG_OF_THE_RAISE_OVER_THE_LAST_VERSION,
        FLOOR_RAISED_PAST_ONE_POSSIBLE_RING_OVER_THE_SLOT_OF_THE_LAST_VERSION,
    );
    let Some(comparison) = compare_the_flow_on_every_state(TEST_NAME, &flow) else {
        return;
    };
    let verdicts = &comparison.rollback_floor_verdicts_of_the_cpu;
    assert!(
        verdicts.iter().any(|verdict| matches!(
            verdict,
            RollbackFloorVerdictOfAState::PartiallyJudged(unjudged)
                if unjudged.contains("判得不一致")
                    && unjudged.contains("可能的环 2 种各判一次：成立 1、F 高于上限 1")
        )) && !verdicts
            .iter()
            .any(|verdict| matches!(verdict, RollbackFloorVerdictOfAState::Violated(_))),
        "流 {} 上 CPU 至少一个状态 I-7.9 报两种可能的环判得不一致、一个状态都不红它（逐状态：{verdicts:?}）",
        flow.name
    );
    assert_the_gpu_agrees_with_the_cpu_on_every_state(&flow, &comparison);
}

// ───────────────── 第 0 代根被抬 F 盖掉、往下认两层以上（判决 `research/prompts/m3-close-b19-reason-r2-main-verification.md` 第二节 Q3 第 3、4 问） ─────────────────

/// 新池新建文件那条流的流尾抄一条 (1, 3) 的根写成 txg 24：24 mod 3 = 0、(24 div 3) mod 8 = 0，正好盖掉区域 0 槽 0 的第 0 代根，把 F 从 0 抬上去。
const TXG_OF_THE_RAISE_OVER_THE_SLOT_OF_THE_GENESIS_ROOT: u64 = 24;
/// 抬到的 F：抬 F 那一刻区域 0 槽 0 住着第 0 代根（树表 0 条），有效根 0–3、非空的只有 3，上限取最旧有效根 0，F 1 越过它。
const FLOOR_RAISED_OVER_THE_SLOT_OF_THE_GENESIS_ROOT: u64 = 1;

/// 第 0 代根被抬 F 盖掉：新池新建文件那条流
/// （根环 txg 0 种子、1、2 暖机、(1, 3) 新建文件）的流尾写一条抄 (1, 3) 的根，txg 24、F 0 → 1、不带卸载记号，盖掉区域 0 槽 0 的第 0 代根。
/// 窗口（txg > 0）里之前的根 1、2、3，非空的只有 3，定不住，走丁；区域 0 槽 0 比 24 小的正 txg 没有、槽 0 而抬之前的 F 是 0：只算第 0 代根一种
/// （两棵用户可见树都没有）。上限 0，F 1 越过它，判红。
/// CPU 至少一个状态上按重建的环红 I-7.9、最旧有效根是 txg 0，GPU 逐状态相同。
#[test]
fn the_gpu_judge_reddens_a_floor_raise_over_the_slot_of_the_genesis_root_past_the_oldest_valid_root_like_the_cpu(
) {
    const TEST_NAME: &str = "the_gpu_judge_reddens_a_floor_raise_over_the_slot_of_the_genesis_root_past_the_oldest_valid_root_like_the_cpu";
    let mut flow = record_new_pool_file_creation(
        "crash-judge-gpu-raise-over-the-genesis-slot",
        "new-pool-floor-raise-over-the-slot-of-the-genesis-root",
    );
    assert_eq!(
        judged_root_identity(&flow),
        PUBLISH_ROOT_IDENTITY,
        "被判的根是新建文件那次发布"
    );
    assert_eq!(
        singlefs_core::root_ring::target_for_publish(
            CheckpointTxg(TXG_OF_THE_RAISE_OVER_THE_SLOT_OF_THE_GENESIS_ROOT),
            geometry().root_ring_slots_per_region,
        )
        .slot,
        0,
        "txg 24 落在区域 0 的槽 0：盖掉的是第 0 代根"
    );
    append_a_copy_of_the_judged_root_at_a_later_txg(
        &mut flow,
        TXG_OF_THE_RAISE_OVER_THE_SLOT_OF_THE_GENESIS_ROOT,
        FLOOR_RAISED_OVER_THE_SLOT_OF_THE_GENESIS_ROOT,
        UnmountMarkerOfTheAppendedRoot::NotWritten,
    );
    let Some(comparison) = compare_the_flow_on_every_state(TEST_NAME, &flow) else {
        return;
    };
    let verdicts = &comparison.rollback_floor_verdicts_of_the_cpu;
    assert!(
        verdicts.iter().any(|verdict| matches!(
            verdict,
            RollbackFloorVerdictOfAState::Violated(detail)
                if detail.contains("按 journal 重建") && detail.contains("最旧有效根 txg 0")
        )) && !verdicts
            .iter()
            .any(|verdict| matches!(verdict, RollbackFloorVerdictOfAState::PartiallyJudged(_))),
        "流 {} 上 CPU 至少一个状态按 journal 重建的环红 I-7.9、最旧有效根 txg 0，一个状态都不报射程里判不了（逐状态：{verdicts:?}）",
        flow.name
    );
    assert_the_gpu_agrees_with_the_cpu_on_every_state(&flow, &comparison);
}

/// 往下认两层那条流（
/// 照这一份测试文件的整池镜像改写）：实例 1 新池新建文件（txg 1、2 暖机，3 新建文件）之后不卸载、关掉镜像重开可写挂载（实例 2：写行 txg 4 落盘 1、
/// 暖机 txg 5 落盘 0，两块盘都有了就不再暖机），实例 2 覆盖写 22 次（txg 6–27，27 = 3 + 3S 与 txg 3 同落区域 0 槽 1），再不卸载、
/// 关掉镜像重开可写挂载（实例 3：写行 txg 28 落盘 1、暖机 txg 29 落盘 0）。
/// 实例 3 的实例表里实例 1 那一行 T = 3、实例 2 那一行 T = 27（或已被整轮清扫删掉——没有行与 u = T 同一处置）。
const OVERWRITES_OF_THE_SECOND_INSTANCE: u64 = 22;
const LAST_TXG_OF_THE_FIRST_INSTANCE: u64 = 3;
const LAST_TXG_OF_THE_SECOND_INSTANCE: u64 = 27;
/// 流尾抄实例 3 最后那条根：txg 30–50 带 F 0（不抬），txg 51 = 27 + 3S 把 F 抬上去、盖掉区域 0 槽 1。
const FIRST_TXG_APPENDED_BEFORE_THE_RAISE_OVER_TWO_LAYERS: u64 = 30;
const TXG_OF_THE_RAISE_OVER_TWO_LAYERS: u64 = 51;
/// 重建的抬 F 那一刻的环：区域 0 槽 1 可能住 txg 27（实例 2 的最后一版，u = T）、txg 3（实例 1 的最后一版，u = T）、空着（比 3 小的正 txg 没有，槽 1 不是槽 0）三种，
/// 别的槽住着 28–50。住 27 那一种上限 27、住 3 那一种上限 3、空着那一种上限 28：F 3 三种都成立；F 4 住 3 那一种红、另两种成立，判得不一致。
const FLOOR_RAISED_WITHIN_EVERY_LAYER: u64 = 3;
const FLOOR_RAISED_PAST_THE_DEEPER_LAYER: u64 = 4;

/// 第 `overwrite_index` 次覆盖写的内容（每次都不同，覆盖写之后每一版用户可见的状态都不同）。
fn content_of_the_overwrite_of_the_second_instance(overwrite_index: u64) -> Vec<u8> {
    (0..FILE_BYTES)
        .map(|index| {
            u8::try_from(
                (u64::try_from(index).expect("下标装得进 u64") * 11 + 5 + overwrite_index) % 251,
            )
            .expect("小于 256")
        })
        .collect()
}

/// 关掉镜像（不卸载：相当于进程崩在最后那次发布之后）重开可写挂载，池换成那次挂载交回的分配器与现行版本（要带文件）；交回挂载取到的实例代号。
fn reopen_writable_without_unmounting(pool: &mut common::BuiltPool, name: &str) -> u32 {
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices)
        .unwrap_or_else(|refused| panic!("流 {name}：不卸载重开可写挂载被拒：{refused:?}"));
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .unwrap_or_else(|| panic!("流 {name}：重开之后现行那一版带文件"));
    pool.output.root.instance.0
}

/// 往下认两层那条流（见 [`OVERWRITES_OF_THE_SECOND_INSTANCE`] 那一段）：被判的根是流里最后一条根槽写（实例 3 暖机那条）。前提逐条钉：
/// 根槽写依次的身份、实例 1 txg 3 与实例 2 txg 27 两版在 journal 里有带提交标记的记录。
fn record_three_instances_whose_last_versions_share_one_ring_slot(
    tag: &str,
    name: &'static str,
) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut content_of_each_txg: Vec<(u64, Vec<u8>)> =
        vec![(LAST_TXG_OF_THE_FIRST_INSTANCE, file_content())];
    assert_eq!(
        reopen_writable_without_unmounting(&mut pool, name),
        2,
        "流 {name}：第一次重开取到实例 2"
    );
    for overwrite_index in 0..OVERWRITES_OF_THE_SECOND_INSTANCE {
        let content = content_of_the_overwrite_of_the_second_instance(overwrite_index);
        pool.output = {
            let publish_parameters = parameters();
            let devices = pool.devices.as_mut().expect("镜像还开着");
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            publish_overwrite(
                &mut writer,
                &mut pool.allocator,
                &pool.output,
                FirstFile {
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 * (overwrite_index + 1),
                },
                InstanceGeneration(2),
            )
            .unwrap_or_else(|refused| {
                panic!("流 {name}：实例 2 第 {overwrite_index} 次覆盖写被拒：{refused:?}")
            })
        };
        content_of_each_txg.push((pool.output.root.checkpoint_txg.0, content));
    }
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(LAST_TXG_OF_THE_SECOND_INSTANCE),
        "流 {name}：实例 2 最后一次覆盖写是 txg 27（与 txg 3 隔一圈 3S = 24）"
    );
    assert_eq!(
        reopen_writable_without_unmounting(&mut pool, name),
        3,
        "流 {name}：第二次重开取到实例 3"
    );
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let entry_spans: Vec<RecordedEntrySpan> = pool
        .stream
        .entry_spans()
        .into_iter()
        .map(|span| RecordedEntrySpan {
            entry: span.entry,
            operations: span.operations.start - pool.mkfs_operation_count
                ..span.operations.end - pool.mkfs_operation_count,
        })
        .collect();
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &operations[pool.mkfs_operation_count..],
        &entry_spans,
        &geometry(),
    );
    let root_identities: Vec<(u32, u64)> = writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .map(|write| root_identity_of_the_root_record(write.bytes().expect("根槽写带字节")))
        .collect();
    let expected_root_identities: Vec<(u32, u64)> = (1..=LAST_TXG_OF_THE_FIRST_INSTANCE)
        .map(|checkpoint_txg| (1, checkpoint_txg))
        .chain(
            (LAST_TXG_OF_THE_FIRST_INSTANCE + 1..=LAST_TXG_OF_THE_SECOND_INSTANCE)
                .map(|checkpoint_txg| (2, checkpoint_txg)),
        )
        .chain(
            (LAST_TXG_OF_THE_SECOND_INSTANCE + 1
                ..FIRST_TXG_APPENDED_BEFORE_THE_RAISE_OVER_TWO_LAYERS)
                .map(|checkpoint_txg| (3, checkpoint_txg)),
        )
        .collect();
    assert_eq!(
        root_identities, expected_root_identities,
        "流 {name} 的根槽写依次是实例 1 暖机两代与新建文件、实例 2 写行暖机两次与 22 次覆盖写、实例 3 写行暖机两次（写行之后暖机到每块盘都有一条为止；对不上就照打出的身份改常量）"
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("挂载写了根槽");
    let content_of = |checkpoint_txg: u64| {
        content_of_each_txg
            .iter()
            .rev()
            .find(|(written_txg, _)| *written_txg <= checkpoint_txg)
            .map(|(_, content)| content.clone())
            .expect("txg 3 起每一版都带文件")
    };
    let versions: Vec<PublishedVersion> = root_identities
        .iter()
        .filter(|(_, checkpoint_txg)| *checkpoint_txg >= LAST_TXG_OF_THE_FIRST_INSTANCE)
        .map(|(instance, checkpoint_txg)| PublishedVersion {
            instance: InstanceGeneration(*instance),
            checkpoint_txg: CheckpointTxg(*checkpoint_txg),
            content: content_of(*checkpoint_txg),
        })
        .collect();
    let flow = RecordedFlow {
        name,
        base,
        writes,
        segments,
        judged_root_index,
        versions,
    };
    for (instance, checkpoint_txg) in [
        (1, LAST_TXG_OF_THE_FIRST_INSTANCE),
        (2, LAST_TXG_OF_THE_SECOND_INSTANCE),
    ] {
        assert!(
            has_a_committed_journal_record_of(&flow, instance, checkpoint_txg),
            "流 {name}：实例 {instance} txg {checkpoint_txg} 那次发布在 journal 里有带提交标记的记录（重建抬 F 那一刻的环按它认那一层）"
        );
    }
    flow
}

/// 往下认两层那条流的流尾抄实例 3 最后那条根：txg 31–50 带 F 0，txg 51 把 F 抬到 `raised_floor`、盖掉区域 0 槽 1。交回流与第一个流尾段的下标。
fn record_a_floor_raise_over_two_layers_of_one_ring_slot(
    tag: &str,
    name: &'static str,
    raised_floor: u64,
) -> (RecordedFlow, usize) {
    let mut flow = record_three_instances_whose_last_versions_share_one_ring_slot(tag, name);
    assert_eq!(
        TXG_OF_THE_RAISE_OVER_TWO_LAYERS - ROOT_RING_SLOT_COUNT_OF_THE_NEW_POOL,
        LAST_TXG_OF_THE_SECOND_INSTANCE,
        "抬 F 那条根盖掉的是实例 2 最后一版那一槽"
    );
    assert_eq!(
        LAST_TXG_OF_THE_SECOND_INSTANCE - ROOT_RING_SLOT_COUNT_OF_THE_NEW_POOL,
        LAST_TXG_OF_THE_FIRST_INSTANCE,
        "再往下一圈是实例 1 最后一版"
    );
    let first_appended_segment = flow.segments.len();
    for checkpoint_txg in
        FIRST_TXG_APPENDED_BEFORE_THE_RAISE_OVER_TWO_LAYERS..TXG_OF_THE_RAISE_OVER_TWO_LAYERS
    {
        append_a_copy_of_the_judged_root_at_a_later_txg(
            &mut flow,
            checkpoint_txg,
            0,
            UnmountMarkerOfTheAppendedRoot::NotWritten,
        );
    }
    append_a_copy_of_the_judged_root_at_a_later_txg(
        &mut flow,
        TXG_OF_THE_RAISE_OVER_TWO_LAYERS,
        raised_floor,
        UnmountMarkerOfTheAppendedRoot::NotWritten,
    );
    (flow, first_appended_segment)
}

/// 往下认两层、三种可能的环都成立的那张：F 抬到 3（不高于三种可能的上限 27、3、28），判成立。CPU 至少一个状态上判过 I-7.9 而成立、
/// 一个状态都不红它、一个状态都不报射程里判不了，GPU 逐状态相同。往下只认一层（认到 27 就停、报认不出）的那一边报射程里判不了，对不上。
#[test]
fn the_gpu_judge_holds_a_floor_raise_over_a_slot_recognized_two_layers_down_within_every_possible_ring_like_the_cpu(
) {
    const TEST_NAME: &str = "the_gpu_judge_holds_a_floor_raise_over_a_slot_recognized_two_layers_down_within_every_possible_ring_like_the_cpu";
    let (flow, first_appended_segment) = record_a_floor_raise_over_two_layers_of_one_ring_slot(
        "crash-judge-gpu-raise-over-two-layers-within-every-ring",
        "three-instance-floor-raise-over-two-layers-within-every-possible-ring",
        FLOOR_RAISED_WITHIN_EVERY_LAYER,
    );
    let Some(comparison) =
        compare_the_flow_expanding_the_segments_from(TEST_NAME, &flow, first_appended_segment)
    else {
        return;
    };
    let verdicts = &comparison.rollback_floor_verdicts_of_the_cpu;
    assert!(
        verdicts.contains(&RollbackFloorVerdictOfAState::Holds)
            && !verdicts.iter().any(|verdict| matches!(
                verdict,
                RollbackFloorVerdictOfAState::Violated(_)
                    | RollbackFloorVerdictOfAState::PartiallyJudged(_)
            )),
        "流 {} 上 CPU 判过 I-7.9 而成立、一个状态都不红它、一个状态都不报射程里判不了（逐状态：{verdicts:?}）",
        flow.name
    );
    assert_the_gpu_agrees_with_the_cpu_on_every_state(&flow, &comparison);
}

/// 往下认两层、三种可能的环判得不一致的那张：F 抬到 4，住 txg 3 那一种上限 3、红，住 27 与空着那两种成立，射程里判不了。
/// CPU 至少一个状态这样报（三种各判一次：成立 2、F 高于上限 1）、一个状态都不红它，GPU 逐状态相同（那一条判到位为 0）。
/// 往下认到第二层不再往下（漏了空着那一种）的那一边报的种数不同、结论照旧不一致；漏了第二层的那一边两种都成立、判成立，对不上。
#[test]
fn the_gpu_judge_leaves_a_floor_raise_past_the_deeper_layer_of_a_slot_recognized_two_layers_down_partially_judged_like_the_cpu(
) {
    const TEST_NAME: &str = "the_gpu_judge_leaves_a_floor_raise_past_the_deeper_layer_of_a_slot_recognized_two_layers_down_partially_judged_like_the_cpu";
    let (flow, first_appended_segment) = record_a_floor_raise_over_two_layers_of_one_ring_slot(
        "crash-judge-gpu-raise-over-two-layers-past-the-deeper-layer",
        "three-instance-floor-raise-past-the-deeper-layer-of-two",
        FLOOR_RAISED_PAST_THE_DEEPER_LAYER,
    );
    let Some(comparison) =
        compare_the_flow_expanding_the_segments_from(TEST_NAME, &flow, first_appended_segment)
    else {
        return;
    };
    let verdicts = &comparison.rollback_floor_verdicts_of_the_cpu;
    assert!(
        verdicts.iter().any(|verdict| matches!(
            verdict,
            RollbackFloorVerdictOfAState::PartiallyJudged(unjudged)
                if unjudged.contains("判得不一致")
                    && unjudged.contains("可能的环 3 种各判一次：成立 2、F 高于上限 1")
        )) && !verdicts
            .iter()
            .any(|verdict| matches!(verdict, RollbackFloorVerdictOfAState::Violated(_))),
        "流 {} 上 CPU 至少一个状态 I-7.9 报三种可能的环判得不一致、一个状态都不红它（逐状态：{verdicts:?}）",
        flow.name
    );
    assert_the_gpu_agrees_with_the_cpu_on_every_state(&flow, &comparison);
}
