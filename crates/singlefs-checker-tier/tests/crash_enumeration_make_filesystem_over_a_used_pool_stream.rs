//! checker 档模块：crash、crash_state_enumeration
//! mkfs 自己的崩溃点重放（C656（mkfs 中途崩溃的语义没有条款） 的验收格；三方判决 `research/prompts/m3-close-c656-r3-main-verification.md`
//! 第四节第 1 条：今天的崩溃枚举用例都从 mkfs 之后的池起，没有一条枚举 mkfs 自己）。
//!
//! 流：用过的盘（旧池：mkfs → 取号 1 → 暖机 → 第一个文件 txg 3，最后确认的一版是 (实例 1, txg 3)）上，经内层入口拿一个与旧池 fsid
//! 前 8 字节不同的 fsid（对外入口从随机源取的那一种，前 8 字节撞上的概率按 2⁻⁶⁴ 算，推的）重做 mkfs，枚举 mkfs 录下的每个崩溃状态。
//! 基线是旧池用完之后的整份盘面，写表是 mkfs 那 27 步（段序列 `2+12+1+1+1+4`），与任何从 mkfs 之后起的流都不相交：
//! 多罩的是 mkfs 自己的全部崩溃状态与它在旧池字节上的撕裂态。
//!
//! 判法（D22（单元原子性怎么合成） 已定项 8 第 3 条射程，用户 2026-10-20 定「认，照改写稿写回」）：观察者按作废持久没有分两截——
//! 两块盘的 [0, 根环基址) 清零有一块没持久（作废持久之前）：恢复读回旧池最后确认的一版；
//! 两块都持久了（作废持久之后）：整池拒（`NoValidSystemConfiguration`），或恢复读回 mkfs 的第 0 代空池（实例 0、txg 0、没有文件）。
//! 别的结局（作废之后读回旧池的任何一版、新系统配置与旧池的根并存、别的错）都让观察者 panic，判红。
//! 判决 S2 量过的分法：31 195 = 8（读回最后确认的一版）+ 31 122（整池拒）+ 65（第 0 代空池）；这一版写序上实现员在 harness 一侧另量过一遍
//! （同一份旧池与 fsid，按段逐个摆崩溃状态、撕裂态按撕裂镜像叠，两遍恢复与池级 checker 逐态判）：同样三组数，两遍恢复逐态同一种结局，
//! 池级 checker 只在整池拒的 31 122 态上红、只红 I-8.5。
//! 整池拒的状态 oracle 记「没择到根」、池级 checker 记 I-8.5（至少一块盘带自证过的系统配置） 红：那是条文允许的结局，用例按态数认下（登记行带
//! `red-blocks=judged-by-test`）。读回旧池那几态上记录核对器记「恢复自称的那一版缺单元」（推的，读 `crash::check_records_against`）：
//! 录制流里只有新池的第 0 代发布（实例 0、txg 0），记录核对器按 txg 拿它与恢复落到的旧池那一版（txg 3）比，认它该在，而它的单元写在第 0 段之后、
//! 还没落，那几个槽上是旧池同一处的单元。两个池之间按 txg 比没有意义，这一红也是条文允许的结局（旧池读回最后确认的一版），用例按态数认下。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{build_pool, crash_state_devices, file_content, geometry, parameters};
use singlefs_checker_tier::crash::{
    crash_state_count_with_torn_in_place_overwrites, full_expansion, quick_tier_expansion,
    CrashPointReplayObserverCounts, CrashPointReplaySegmentExpansion, CrashPointReplayTally,
    CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_with_expansion, CrashPointReplayItem, CrashPointReplayObserver,
};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs";
use singlefs_core::address::{CheckpointTxg, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::recovery::{RecoveryFailure, RecoveryOutcome, RecoveryReport};
use singlefs_harness::memory_pool::{
    writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 观察者计数：作废持久之前读回旧池最后确认的一版。
const OLD_POOL_READ_BACK: &str = "old_pool_read_back_its_last_confirmed_version";
/// 观察者计数：作废持久之后整池拒（`NoValidSystemConfiguration`）。
const WHOLE_POOL_REFUSED: &str = "whole_pool_refused_without_a_valid_system_configuration";
/// 观察者计数：作废持久之后恢复读回第 0 代空池。
const GENERATION_ZERO_EMPTY_POOL: &str = "generation_zero_empty_pool_read_back";

/// I-8.5（至少一块盘带自证过的系统配置） 在池级 checker 里的名字：整池拒的状态上它判红。
const SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION: &str = "I-8.5";
/// I-7.7（系统配置实例代号不低于根环） 在池级 checker 里的名字：前 8 字节不同时第 0 代空池上它判成立（C659 那一形不在这条流里）。
const SYSTEM_CONFIGURATION_INSTANCE_INVARIANT: &str = "I-7.7";

struct Prepared {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    /// 旧池最后确认的一版（oracle 的版本表里只有它一版有文件）。
    old_pool_versions: Vec<PublishedVersion>,
    /// 两块盘 [0, 根环基址) 清零那两次写在写表里的下标：两次都持久就是作废持久了。
    invalidation_writes: Vec<usize>,
}

fn identifier_differing_in_the_first_eight_bytes_from_the_old_pool() -> [u8; 16] {
    let mut filesystem_identifier = parameters().filesystem_identifier;
    filesystem_identifier[..8].copy_from_slice(b"remade-x");
    filesystem_identifier
}

fn prepare(tag: &str) -> Prepared {
    let old_pool = build_pool(tag);
    let old_pool_versions = vec![PublishedVersion {
        instance: old_pool.output.root.instance,
        checkpoint_txg: old_pool.output.root.checkpoint_txg,
        content: file_content(),
    }];
    assert_eq!(
        (
            old_pool_versions[0].instance,
            old_pool_versions[0].checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(3)),
        "旧池最后确认的一版是第一个文件那一版"
    );
    let base = old_pool.memory_pool();
    let common_parameters = parameters();
    let remade_parameters = MakeFilesystemParameters {
        filesystem_identifier: identifier_differing_in_the_first_eight_bytes_from_the_old_pool(),
        region_devices: common_parameters.region_devices,
        geometry: common_parameters.geometry,
    };
    assert_ne!(
        remade_parameters.filesystem_identifier[..8],
        common_parameters.filesystem_identifier[..8]
    );
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&base, &[], &[], &stream);
    make_filesystem(&remade_parameters, &mut devices).expect("用过的盘上重做 mkfs");
    let (writes, segments) = writes_and_segments(&stream.retained_operations(), &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<usize>>(),
        vec![2, 12, 1, 1, 1, 4],
        "两趟写序：[0, 根环基址) 清零两写自成一段（作废）| 其余八次清零与四个单元写 | 三条第 0 代根各一段 | 四个系统配置槽"
    );
    let invalidation_writes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::ZeroFill && write.offset == DeviceOffsetInBytes(0)
        })
        .map(|(write_index, _)| write_index)
        .collect();
    assert_eq!(
        invalidation_writes, segments[0],
        "第一段就是两块盘的 [0, 根环基址) 清零"
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("mkfs 写第 0 代根");
    Prepared {
        base,
        writes,
        segments,
        judged_root_index,
        old_pool_versions,
        invalidation_writes,
    }
}

/// 观察者另读的输入（作废那两次写的下标、旧池最后确认的那一版）的摘要：进观察者的版本，录制流一变它就变。
fn observer_inputs_digest(prepared: &Prepared) -> [u8; singlefs_harness::sha256::DIGEST_BYTES] {
    let digest = singlefs_harness::sha256::sha256_digest(
        format!(
            "invalidation_writes={:?} old_pool_versions={:?}",
            prepared.invalidation_writes, prepared.old_pool_versions
        )
        .as_bytes(),
    );
    assert_ne!(
        digest,
        [0; singlefs_harness::sha256::DIGEST_BYTES],
        "观察者另读的输入的摘要真算（全零就是没算）"
    );
    digest
}

/// 一个崩溃状态的结局落在条文允许的哪一种里，记一个数；落不进任何一种就 panic。
fn observe_the_outcome_allowed_by_the_clause(
    prepared: &Prepared,
    image: &CrashImage<'_>,
    report: &RecoveryReport,
    counts: &mut CrashPointReplayObserverCounts,
) {
    let invalidation_is_durable = prepared
        .invalidation_writes
        .iter()
        .all(|write_index| image.persisted[*write_index]);
    let old_version = &prepared.old_pool_versions[0];
    match (invalidation_is_durable, &report.outcome) {
        (false, RecoveryOutcome::FileRead { root, content })
            if *root == (old_version.instance, old_version.checkpoint_txg)
                && *content == old_version.content =>
        {
            counts.add(OLD_POOL_READ_BACK, 1);
        }
        (
            true,
            RecoveryOutcome::Failed {
                failure: RecoveryFailure::NoValidSystemConfiguration { .. },
                ..
            },
        ) => counts.add(WHOLE_POOL_REFUSED, 1),
        (true, RecoveryOutcome::NoFile { root })
            if *root == (InstanceGeneration(0), CheckpointTxg(0)) =>
        {
            counts.add(GENERATION_ZERO_EMPTY_POOL, 1);
        }
        (invalidation_is_durable, outcome) => panic!(
            "mkfs 中途崩溃的结局不在条文允许的里面（作废持久 = {invalidation_is_durable}）：{outcome:?}"
        ),
    }
}

fn enumerate_with_the_clause_observer(
    prepared: &Prepared,
    item_name: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Option<CrashPointReplayTally> {
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        observe_the_outcome_allowed_by_the_clause(prepared, image, report, counts);
    };
    enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: item_name,
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        CrashPointReplayVersionTable::with_a_file_only(&prepared.old_pool_versions),
        expansion,
        &[CrashPointReplayObserver {
            name: "make_filesystem_clause",
            extra_inputs_digest: observer_inputs_digest(prepared),
            observe: &observe,
        }],
    )
    .judged()
}

/// 态数之外的判定：观察者看过每个状态、三种结局的数；oracle 与池级 checker 只在整池拒的状态上红，红的那一种是条文允许的。
fn assert_every_state_lands_in_an_allowed_outcome(
    tally: &CrashPointReplayTally,
    expected_old_pool_states: u64,
    expected_generation_zero_states: u64,
) {
    assert_eq!(tally.observed_states, tally.states, "观察者看过每一个状态");
    let refused_states = tally.observer_counts.count_named(WHOLE_POOL_REFUSED);
    assert_eq!(
        (
            tally.observer_counts.count_named(OLD_POOL_READ_BACK),
            tally
                .observer_counts
                .count_named(GENERATION_ZERO_EMPTY_POOL),
            refused_states,
        ),
        (
            expected_old_pool_states,
            expected_generation_zero_states,
            tally.states - expected_old_pool_states - expected_generation_zero_states,
        ),
        "读回旧池最后确认的一版 / 第 0 代空池 / 整池拒"
    );
    assert_eq!(
        (tally.violations, tally.ignored_violations),
        (refused_states, refused_states),
        "oracle 只在整池拒的状态上记「没择到根」：{:?}",
        tally.first_violation
    );
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, expected_old_pool_states),
        "记录核对器两条判据：第 0 代发布没有 journal 记录，第一条不判；读回旧池的那几态上第二条按 txg 认新池第 0 代的单元该在（文件头）"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        let expected = if invariant == SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION {
            refused_states
        } else {
            0
        };
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            expected,
            "{invariant} 判违例的状态数（只有整池拒的状态上 I-8.5 红）：{:?}",
            tally.checker_first_violation.get(invariant)
        );
    }
    assert!(
        tally
            .checker_evaluated_states
            .get(SYSTEM_CONFIGURATION_INSTANCE_INVARIANT)
            .copied()
            .unwrap_or(0)
            > 0,
        "I-7.7 在第 0 代空池的状态上真被评估过"
    );
}

/// 平时跑的那一份（甲二）：原地写（清零与系统配置槽写）每段任意子集、各取三态或两态，四个单元写全不落或全落。
#[test]
fn every_quick_tier_crash_state_of_make_filesystem_over_a_used_pool_lands_in_an_outcome_the_clause_allows(
) {
    let prepared = prepare("make-filesystem-over-a-used-pool-quick");
    let Some(tally) = enumerate_with_the_clause_observer(
        &prepared,
        "make_filesystem_over_a_used_pool::quick_tier",
        &quick_tier_expansion,
    ) else {
        return;
    };
    assert_eq!(
        tally.states,
        crash_state_count_with_torn_in_place_overwrites(
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            &quick_tier_expansion,
        ),
        "按甲二展开的崩溃点重放枚举域"
    );
    assert_every_state_lands_in_an_allowed_outcome(&tally, 8, 65);
}

/// 全量：每一段任意真子集（原地覆写取三态），判决 S2 量过的 31 195 = 8 + 31 122 + 65。
#[test]
#[ignore = "崩溃枚举用例全量：门禁 checker-tier-crash-point-replay --full 与崩溃点重放入口跑"]
fn full_enumeration_of_make_filesystem_over_a_used_pool_lands_every_crash_state_in_an_outcome_the_clause_allows(
) {
    let prepared = prepare("make-filesystem-over-a-used-pool-full");
    let Some(tally) = enumerate_with_the_clause_observer(
        &prepared,
        "make_filesystem_over_a_used_pool::every_crash_state",
        &full_expansion,
    ) else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY_MAKE_FILESYSTEM_OVER_A_USED_POOL states={} old_pool_read_back={} whole_pool_refused={} generation_zero_empty_pool={} checker_by_invariant(evaluated/violated/not_applicable) {}",
        tally.states,
        tally.observer_counts.count_named(OLD_POOL_READ_BACK),
        tally.observer_counts.count_named(WHOLE_POOL_REFUSED),
        tally.observer_counts.count_named(GENERATION_ZERO_EMPTY_POOL),
        tally.checker_counts_by_invariant()
    );
    assert_eq!(
        tally.states,
        crash_state_count_with_torn_in_place_overwrites(
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            &full_expansion,
        ),
        "全量的崩溃点重放枚举域（原地覆写取三态）"
    );
    assert_eq!(tally.states, 31_195, "判决 S2：1 + 8 + 31 103 + 3 + 80");
    assert_every_state_lands_in_an_allowed_outcome(&tally, 8, 65);
}
