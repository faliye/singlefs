//! 发布的「单元写 → 屏障 → journal 记录」那道屏障（`singlefs_core::transaction` 的 `persist_publish_writes` 里第一道）防住的状态——
//! 记录已持久、它点名的某个单元两份都没落——恢复端还有第二道防线：施加记录之前逐份读点名单元核校验和（`replay_journal` 的点名验证）。
//! 两道至少留一道（K3-⑨，用户 2026-09-28 选；事实见 `research/prompts/k3-9-mechanism/report.md`「四次运行」：删掉那道屏障，
//! 照常点名验证时三条判据 0、验证失败 903 次；再关掉点名验证，记录核对器与模型各红 752 次）。
//! 这里不改 core，用录制器录下一段真实历史，在持久集合里直接摆出这个状态，钉住第二道防线承重：
//! 照常恢复落回上一版、验证失败有数、理想模型不分歧；关掉点名验证（只供测试的开关）时理想模型报分歧。
//!
//! 点名验证替不了这道屏障的那一处也钉在这里（C592，`research/prompts/k3-9-barrier-r1-main-verification.md` 第四节、第六节第 2 题）：
//! 复用回收槽的历史上，池级 checker I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）射程④ 的同源前提靠这道屏障撑着。

use std::collections::BTreeMap;
use std::rc::Rc;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::{
    mount_writable_with_space_admission, raise_rollback_floor, ShadowLedger,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryReport};
use singlefs_core::transaction::{
    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
    PerStepChecker,
};
use singlefs_harness::memory_pool::{
    newest_persisted_root, publishes_in, writes_and_segments, CrashImage, MemoryPool,
    RecordStreamContinuity, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::model::{crash_recovery_disagreement, ModelDisagreement, ModelRootKey};
use singlefs_harness::model_comparison::{model_root_key, observed_read_back_after_a_crash};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 不跑每一步的池级 checker（这里只看崩溃之后那一个状态），两块 4 GiB 的盘。
const EXECUTION: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::FourGibibytes,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

/// 起点发完第一个文件 (1, 3)，再覆盖写一次 (1, 4)：被摆的是 (1, 4) 那次发布。
const THE_PUBLISH_BEFORE: (InstanceGeneration, CheckpointTxg) =
    (InstanceGeneration(1), CheckpointTxg(3));
const THE_PUBLISH_WHOSE_UNIT_NEVER_LANDED: (InstanceGeneration, CheckpointTxg) =
    (InstanceGeneration(1), CheckpointTxg(4));

/// 录下来的这段历史与摆好的那一个崩溃状态：写表、持久集合（与写表逐条对应）、模型提交过的每一版。
struct RecordPersistedWithoutOneOfItsUnits {
    writes: Vec<RetainedWrite>,
    persisted: Vec<bool>,
    committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
}

/// 跑这段历史、录下整条流，摆出：(1, 4) 的根槽写之前的写都持久，只有它第一个单元写那个偏移上的每一份（两块盘各一份）扣下；
/// 它的根槽写与之后的都没持久。于是 (1, 4) 的 journal 记录两份都在盘上，而它点名的一个单元两份都不在——
/// 这正是记录之前那道屏障守住时摆不出来的状态（屏障之前的单元写没落，屏障之后的记录就不会落）。
fn record_persisted_without_one_of_its_units() -> RecordPersistedWithoutOneOfItsUnits {
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 5 },
            fill_seed: 5,
        })],
    };
    let stream = SharedStream::retaining_contents();
    let mut committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = BTreeMap::new();
    let run = execute_history_with(&history, EXECUTION, &stream, &mut |observation| {
        for (key, file) in observation.model.committed_versions() {
            committed_versions.insert(key, file);
        }
    });
    assert_eq!(run.ending, HistoryEnding::Completed, "这段历史跑完");
    let (writes, _segments) = writes_and_segments(
        &stream.retained_operations(),
        &EXECUTION.device_width.fixed_geometry(),
    );
    let every_write = vec![true; writes.len()];
    let publishes = publishes_in(&writes, &every_write, RecordStreamContinuity::OneRecording);
    let publish = publishes
        .iter()
        .find(|publish| {
            (
                InstanceGeneration(publish.instance),
                CheckpointTxg(publish.checkpoint_txg),
            ) == THE_PUBLISH_WHOSE_UNIT_NEVER_LANDED
        })
        .unwrap_or_else(|| {
            panic!(
                "流里有 {THE_PUBLISH_WHOSE_UNIT_NEVER_LANDED:?} 那次发布：{:?}",
                publishes
                    .iter()
                    .map(|publish| (publish.instance, publish.checkpoint_txg))
                    .collect::<Vec<_>>()
            )
        });
    assert!(
        !publish.units.is_empty() && !publish.records.is_empty(),
        "覆盖写那次发布写了单元，也写了 journal 记录"
    );
    let withheld_offset = writes[publish.units[0]].offset;
    let mut persisted = vec![false; writes.len()];
    persisted[..publish.root].fill(true);
    let mut withheld_copies = 0usize;
    for unit in &publish.units {
        if writes[*unit].offset == withheld_offset {
            persisted[*unit] = false;
            withheld_copies += 1;
        }
    }
    assert_eq!(withheld_copies, 2, "扣下的是同一个单元在两块盘上的两份");
    assert!(
        publish.records.iter().all(|record| persisted[*record]),
        "那次发布的 journal 记录每一份都持久"
    );
    RecordPersistedWithoutOneOfItsUnits {
        writes,
        persisted,
        committed_versions,
    }
}

/// 在这个崩溃状态上按 `policy` 恢复，交回恢复报告与理想模型的判定（盘上已持久的最新根槽取持久集合里的）。
fn recover_and_ask_the_model(
    state: &RecordPersistedWithoutOneOfItsUnits,
    policy: JournalPolicy,
) -> (RecoveryReport, Option<ModelDisagreement>) {
    let empty_devices = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        EXECUTION.device_width.device_bytes(),
    );
    let image = CrashImage {
        base: &empty_devices,
        writes: &state.writes,
        persisted: state.persisted.clone(),
    };
    let report = recover(&image, policy);
    let newest_persisted = newest_persisted_root(&state.writes, &state.persisted)
        .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
    assert_eq!(
        newest_persisted,
        Some(model_root_key(THE_PUBLISH_BEFORE.0, THE_PUBLISH_BEFORE.1)),
        "盘上已持久的最新根槽是上一版：被摆的那次发布的根槽写没持久"
    );
    let disagreement = crash_recovery_disagreement(
        &state.committed_versions,
        newest_persisted,
        &observed_read_back_after_a_crash(&report),
    );
    (report, disagreement)
}

/// 照常恢复（点名验证开着）：那条记录点名的单元读不出，验证失败、这次发布整体不施加，恢复落回上一版，理想模型认。
/// 把点名验证短路（恢复交给前缀核的开关恒为「不验」，`crates/mutations.tsv`）就施加了那条记录、走读在缺的单元上失败，这一条红。
#[test]
fn with_named_unit_verification_a_record_whose_unit_never_landed_falls_back_to_the_previous_version(
) {
    let state = record_persisted_without_one_of_its_units();
    let (report, disagreement) = recover_and_ask_the_model(&state, JournalPolicy::Consult);
    assert!(
        report.journal.verification_failed > 0,
        "点名验证读到了缺的那个单元、判了失败：{:?}",
        report.journal
    );
    assert_eq!(
        report.journal.prefix_applied, 0,
        "验证失败的那次发布整体不施加：{:?}",
        report.journal
    );
    assert_eq!(
        report.effective_root,
        Some(THE_PUBLISH_BEFORE),
        "恢复落回上一版：{:?}",
        report.outcome
    );
    assert_eq!(
        disagreement, None,
        "理想模型认上一版（盘上已持久的最新根槽就是它）"
    );
}

/// 关掉点名验证（只供测试的开关）：同一个状态上记录被施加、恢复自称落到那次发布，走读在缺的单元上失败，理想模型报分歧——
/// 两道防线都拿掉时这个状态就是一次错的恢复，上一条的「落回上一版」确实是点名验证挡下来的。
#[test]
fn without_named_unit_verification_the_same_state_is_a_model_disagreement() {
    let state = record_persisted_without_one_of_its_units();
    let (report, disagreement) =
        recover_and_ask_the_model(&state, JournalPolicy::ConsultWithoutNamedVerification);
    assert_eq!(
        (report.journal.prefix_applied > 0, report.effective_root),
        (true, Some(THE_PUBLISH_WHOSE_UNIT_NEVER_LANDED)),
        "不验就施加了那条记录、恢复自称落到那次发布：{:?}",
        report.journal
    );
    assert!(
        disagreement.is_some(),
        "理想模型报分歧（恢复走读失败或读错）：{:?}",
        report.outcome
    );
}

/// 复用回收槽的那段历史（攻方 `research/prompts/k3-9-barrier-r1-opus-model/` 的 floorreuse4 场景）：两块单元区 384 槽的小盘，
/// mkfs、挂载、第一个文件、覆盖写 4 次、抬 F 到上限（现行 txg − 3），抬 F 之后换下的槽可回收；再覆盖写一次（被判的那一次），
/// 它的单元落在盘上已有旧单元的槽里。交回被判那一次之前的池（之前的写全持久）、被判那一次的写表与切段。
struct FloorRaisedThenAPublishReusingReclaimedSlots {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
}

/// 第 `round` 次发布的内容：长度不同、字节不同，每次覆盖写都换下上一版的数据单元。
fn content_of_round(round: usize, length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 13 + round * 7 + 3) % 251).expect("小于 251"))
        .collect()
}

/// 每次发布的写入时间：照 E142 那个固定时刻起，每次往后一秒。
const FIRST_WRITE_TIME_SECONDS: u64 = 1_788_000_000;

fn floor_raised_then_a_publish_reusing_reclaimed_slots(
) -> FloorRaisedThenAPublishReusingReclaimedSlots {
    let width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let parameters = width.parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        SparseBlockDevice::new(
                            width.device_bytes(),
                            PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
                        ),
                        stream.clone(),
                    ),
                )
            })
            .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut mounted = mount_writable_with_space_admission(
        &parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("mkfs 之后的可写挂载");
    let PoolVersion::WithoutFile(version_without_file) = &mounted.current else {
        panic!("mkfs 之后挂上的那一版还没有文件");
    };
    let root_without_file = version_without_file.root;
    let record_bytes_without_file = version_without_file.record_bytes.clone();
    let instance = mounted.output.instance;
    let first_content = content_of_round(1, 3000);
    let mut current = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut mounted.allocator,
            &root_without_file,
            FirstFile {
                content: &first_content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS,
            },
            instance,
            &record_bytes_without_file,
        )
        .expect("第一个文件")
    };
    for round in 0..4u64 {
        let round_index = usize::try_from(round).expect("小于 4");
        let content = content_of_round(round_index + 10, 3000 + round_index);
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        current = publish_overwrite(
            &mut writer,
            &mut mounted.allocator,
            &current,
            FirstFile {
                content: &content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 1 + round,
            },
            instance,
        )
        .expect("覆盖写");
    }
    // 抬 F 的上限是现行 txg − 3（攻方先试时 `RollbackFloorAboveCeiling` 报的 ceiling）。
    let floor_at_the_ceiling = CheckpointTxg(current.root.checkpoint_txg.0 - 3);
    raise_rollback_floor(
        &parameters,
        &mut devices,
        &mut mounted.allocator,
        &mut current,
        floor_at_the_ceiling,
        ShadowLedger::On,
    )
    .expect("抬 F 到上限");
    let operations_before_the_judged_publish = stream.operation_count();
    let judged_content = content_of_round(2, 4000);
    {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut mounted.allocator,
            &current,
            FirstFile {
                content: &judged_content,
                write_time_seconds: FIRST_WRITE_TIME_SECONDS + 5,
            },
            instance,
        )
        .expect("被判的那一次覆盖写");
    }
    let operations = stream.retained_operations();
    let mut base = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        width.device_bytes(),
    );
    base.apply(&operations[..operations_before_the_judged_publish]);
    let (writes, segments) = writes_and_segments(
        &operations[operations_before_the_judged_publish..],
        &width.fixed_geometry(),
    );
    FloorRaisedThenAPublishReusingReclaimedSlots {
        base,
        writes,
        segments,
    }
}

/// C592（单独拿掉单元写与记录之间那道屏障，今天没有判据会红）的那一格：复用回收槽的历史上，被判那一次发布的每一段各扣下一个写
/// （更早的段整段持久、这一段别的写持久、更晚的段一个都没持久），池级 checker 一条不红。这道屏障拿掉之后单元写与记录并成一段，
/// 「复用槽上那个数据单元的一份没落、记录落了」那个状态进了枚举域，下一次挂载会先施加的那一版的分配记录与它罩住的单元不同源，
/// I-3.10 在结局正确的状态上红（攻方 floorreuse4：删屏障 271 个状态红 48，带屏障 73 个状态 0；`crates/mutations.tsv` 删屏障那一行点名这一条）。
// crash-case-check:not-a-crash-case 只摆被判那一次发布的单扣状态，几十个，debug 下单跑几秒
#[test]
fn with_the_barrier_before_the_record_a_publish_reusing_reclaimed_slots_keeps_every_single_withheld_state_free_of_checker_violations(
) {
    let judged = floor_raised_then_a_publish_reusing_reclaimed_slots();
    let reused_unit_writes = judged
        .writes
        .iter()
        .filter(|write| {
            write.kind == StepKind::UnitWrite
                && !judged.base.devices[&write.device]
                    .written_sectors_in(write.offset, write.length_in_bytes())
                    .is_empty()
        })
        .count();
    assert!(
        reused_unit_writes >= 1,
        "前提：被判那一次至少有一个单元写落在盘上已写过的槽里（复用回收的槽）"
    );
    let mut states_judged = 0usize;
    let mut violations_found: Vec<String> = Vec::new();
    // 迭代上界是被判那一次发布的写数；每一轮从同一份基线起摆一个状态，不跨轮带状态。
    for (segment_index, segment) in judged.segments.iter().enumerate() {
        for withheld in segment {
            let mut persisted = vec![false; judged.writes.len()];
            for earlier_segment in &judged.segments[..segment_index] {
                for write in earlier_segment {
                    persisted[*write] = true;
                }
            }
            for write in segment {
                persisted[*write] = write != withheld;
            }
            let image = CrashImage {
                base: &judged.base,
                writes: &judged.writes,
                persisted,
            };
            states_judged += 1;
            for (invariant, verdict) in check_pool_image(&image) {
                if let InvariantVerdict::Violated(detail) = verdict {
                    violations_found.push(format!(
                        "第 {segment_index} 段扣下第 {withheld} 个写：{invariant} {detail}"
                    ));
                }
            }
        }
    }
    assert_eq!(
        states_judged,
        judged.writes.len(),
        "每个写各扣下一次，状态数等于写数"
    );
    assert!(
        violations_found.is_empty(),
        "带着单元写与记录之间那道屏障，单扣状态上池级 checker 红了：{violations_found:#?}"
    );
}
