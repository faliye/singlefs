//! 发布的「单元写 → 屏障 → journal 记录」那道屏障（`singlefs_core::transaction` 的 `persist_publish_writes` 里第一道）防住的状态——
//! 记录已持久、它点名的某个单元两份都没落——恢复端还有第二道防线：施加记录之前逐份读点名单元核校验和（`replay_journal` 的点名验证）。
//! 两道至少留一道（K3-⑨，用户 2026-09-28 选；事实见 `research/prompts/k3-9-mechanism/report.md`「四次运行」：删掉那道屏障，
//! 照常点名验证时三条判据 0、验证失败 903 次；再关掉点名验证，记录核对器与模型各红 752 次）。
//! 这里不改 core，用录制器录下一段真实历史，在持久集合里直接摆出这个状态，钉住第二道防线承重：
//! 照常恢复落回上一版、验证失败有数、理想模型不分歧；关掉点名验证（只供测试的开关）时理想模型报分歧。
//!
//! 点名验证替不了这道屏障的那一处也钉在这里（C592，`research/prompts/k3-9-barrier-r1-main-verification.md` 第四节、第六节第 2 题）：
//! 复用回收槽的历史上，池级 checker I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）射程④ 的同源前提靠这道屏障撑着。
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/named_unit_verification_stands_in_for_the_barrier_before_the_record.rs"]
mod scenarios_of_named_unit_verification_stands_in_for_the_barrier_before_the_record;

use common_images_for_the_checker_tier::PoolCheckerImages;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryReport};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
    PerStepChecker,
};
use singlefs_harness::memory_pool::{
    newest_persisted_root, publishes_in, writes_and_segments, CrashImage, MemoryPool,
    RecordStreamContinuity, RetainedWrite,
};
use singlefs_harness::model::{crash_recovery_disagreement, ModelDisagreement, ModelRootKey};
use singlefs_harness::model_comparison::{model_root_key, observed_read_back_after_a_crash};
use singlefs_harness::SharedStream;
use std::collections::BTreeMap;
use std::rc::Rc;

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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_named_unit_verification_stands_in_for_the_barrier_before_the_record::with_the_barrier_before_the_record_a_publish_reusing_reclaimed_slots_keeps_every_single_withheld_state_free_of_checker_violations`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn with_the_barrier_before_the_record_a_publish_reusing_reclaimed_slots_keeps_every_single_withheld_state_free_of_checker_violations(
) {
    scenarios_of_named_unit_verification_stands_in_for_the_barrier_before_the_record::with_the_barrier_before_the_record_a_publish_reusing_reclaimed_slots_keeps_every_single_withheld_state_free_of_checker_violations(&mut PoolCheckerImages::left_to_the_checker_tier());
}
