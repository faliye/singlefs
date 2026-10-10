//! 第二截与二次崩溃（照 `crash_injection.rs` 的写法，用它公开的函数）：崩溃镜像上可写挂载一次再发一次布；
//! 被拒的按 D23 单子复算拒因、注没注入读故障两种标记各判一次；做成的判挂载透明（只挂载、挂载加一次发布），
//! 再在挂载那一段写表上逐段摆二次崩溃（段内真子集，写数多的段按种子抽），每个二次崩溃镜像判挂载透明。
use std::collections::BTreeMap;

use singlefs_checker_tier::crash_injection::{
    mount_and_publish_once_on_the_crash_image, CrashPoint, WritableMountOutcome,
};
use singlefs_checker_tier::writable_mount_phases::materialized_crash_image;
use singlefs_harness::history::{HistorySeed, StepPosition};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, CrashImage, RetainedWrite,
};
use singlefs_harness::mount_transparency::{
    judge_mount_transparency, judge_writable_mount_refusal_of_a_crash_state, read_recovery_fields,
    refusal_reason_on_the_d23_list, ComparedVersion, DiskAfterTheRefusal, FieldOutcome,
    MountTransparency, MountTransparencyAction, ReadFaultsInTheCrashState,
    ReadOnlyReadBackOfTheCrashImage, RecoveryFieldsRead, WritableMountRefusalVerdict,
};

use crate::common::Recorded;

pub struct StateRecord {
    pub persisted: Vec<bool>,
    pub segment: usize,
    pub injected: bool,
}

#[derive(Default)]
pub struct Tally {
    pub counts: BTreeMap<String, u64>,
    pub examples: BTreeMap<String, Vec<String>>,
}

impl Tally {
    pub fn add(&mut self, key: String) {
        *self.counts.entry(key).or_insert(0) += 1;
    }
    pub fn example(&mut self, key: &str, text: String) {
        let list = self.examples.entry(key.to_string()).or_default();
        if list.len() < 3 {
            list.push(text);
        }
    }
    pub fn absorb(&mut self, other: Tally) {
        for (key, count) in other.counts {
            *self.counts.entry(key).or_insert(0) += count;
        }
        for (key, list) in other.examples {
            for text in list {
                self.example(&key, text);
            }
        }
    }
}

fn record_transparency(tally: &mut Tally, tag: &str, transparency: &MountTransparency, where_: &str) {
    match transparency {
        MountTransparency::NotJudged(reason) => tally.add(format!("mt-not-judged/{tag}/{reason:?}")),
        MountTransparency::Judged(verdicts) => {
            tally.add(format!("mt-judged/{tag}"));
            for verdict in verdicts {
                match &verdict.outcome {
                    FieldOutcome::Holds => {}
                    FieldOutcome::NotJudged(reason) => {
                        tally.add(format!("mt-field-not-judged/{tag}/{reason:?}"))
                    }
                    FieldOutcome::Violated { expected, observed } => {
                        let key = format!("MT-VIOLATED/{tag}/{:?}", verdict.field);
                        tally.add(key.clone());
                        tally.example(&key, format!("{where_}: 期望 {expected}；读出 {observed}"));
                    }
                }
            }
        }
    }
}

fn short(verdict: &WritableMountRefusalVerdict) -> String {
    match verdict {
        WritableMountRefusalVerdict::Legal(reason) => format!("Legal({reason:?})"),
        WritableMountRefusalVerdict::Illegal { reason, conditions_not_met } => {
            format!("Illegal({reason:?},{conditions_not_met:?})")
        }
        WritableMountRefusalVerdict::ReasonNotOnTheD23List => "NotOnList".to_string(),
    }
}

/// 一段里摆哪些二次崩溃：写数 n 的段全部真子集 2^n − 1 个不超过 `cap` 时全摆，否则按种子抽 `cap` 个（含空集）。
fn subsets(n: usize, cap: usize, salt: u64) -> Vec<u64> {
    if n < 63 && (1u64 << n) - 1 <= cap as u64 {
        return (0..(1u64 << n) - 1).collect();
    }
    let mut state = salt ^ 0x9E37_79B9_7F4A_7C15;
    let mut chosen = vec![0u64];
    while chosen.len() < cap {
        state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        let mask = if n >= 64 { state } else { state & ((1u64 << n) - 1) };
        let full = if n >= 64 { u64::MAX } else { (1u64 << n) - 1 };
        if mask != full && !chosen.contains(&mask) {
            chosen.push(mask);
        }
    }
    chosen
}

pub fn judge_state(
    recorded: &Recorded,
    enumerated_writes: &[RetainedWrite],
    state: &StateRecord,
    seed: u64,
    second_crash_cap: usize,
    tally: &mut Tally,
) {
    let crash = materialized_crash_image(&CrashImage {
        base: &recorded.base,
        writes: enumerated_writes,
        persisted: state.persisted.clone(),
    });
    let first: RecoveryFieldsRead = read_recovery_fields(&crash, ComparedVersion::TheRecoveredVersion);
    let tag = if state.injected { "injected" } else { "clean" };
    let crash_point = CrashPoint {
        segment_index: state.segment,
        persisted_within_the_segment: Vec::new(),
        step: StepPosition::StartingPoint,
        operation_kind: None,
    };
    let run = mount_and_publish_once_on_the_crash_image(
        &crash,
        &recorded.parameters,
        recorded.execution,
        HistorySeed(seed),
        &crash_point,
    );
    let where_ = format!("seed {seed} 第一次崩溃段 {} injected={}", state.segment, state.injected);
    tally.add(format!("stage2/{tag}"));
    if let WritableMountOutcome::MountRefused { refusal, member, .. } = &run.outcome {
        let reason = refusal_reason_on_the_d23_list(refusal);
        tally.add(format!("refused/{tag}/{reason:?}"));
        if reason.is_none() {
            let key = format!("refused-not-on-list/{tag}");
            tally.example(&key, format!("{where_}: {}", member.chars().take(300).collect::<String>()));
            return;
        }
        let disk = if run.pool_after == crash {
            DiskAfterTheRefusal::ByteForByteUnchanged
        } else {
            DiskAfterTheRefusal::Changed
        };
        for flag in [ReadFaultsInTheCrashState::NoneInjected, ReadFaultsInTheCrashState::Injected] {
            let verdict = judge_writable_mount_refusal_of_a_crash_state(
                &crash,
                first.report.effective_root,
                refusal,
                disk,
                ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
                flag,
            );
            let key = format!("refusal-verdict/{tag}/judged-as-{flag:?}/{}", short(&verdict));
            tally.add(key.clone());
            tally.example(&key, format!("{where_}: {}", member.chars().take(300).collect::<String>()));
        }
        return;
    }
    if run.rows_deleted_by_the_instance_table_sweep.is_some() {
        tally.add(format!("skipped-sweep-deleted-rows/{tag}"));
        return;
    }
    let (mount_writes, mount_segments, _) =
        writes_and_segments_with_stream_indexes(&run.mount_operations, &recorded.geometry);
    let mut after_the_mount = crash.clone();
    after_the_mount.apply_writes(&mount_writes);
    let after = read_recovery_fields(&after_the_mount, ComparedVersion::TheRecoveredVersion);
    let floor = run.floor_raised_by_the_mount.expect("挂载做成时交回抬 F");
    let continuation = run
        .user_visible_tree_root_pointers_of_the_continuation
        .as_ref()
        .expect("挂载做成时交回续做根指针");
    record_transparency(
        tally,
        &format!("{tag}/alone"),
        &judge_mount_transparency(&first, &after, MountTransparencyAction::MountAlone(floor), continuation),
        &where_,
    );
    if matches!(run.outcome, WritableMountOutcome::Published) {
        let after_publish =
            read_recovery_fields(&run.pool_after, ComparedVersion::TheVersionBeforeTheRecoveredOne);
        record_transparency(
            tally,
            &format!("{tag}/publish"),
            &judge_mount_transparency(
                &first,
                &after_publish,
                MountTransparencyAction::MountAndOnePublish(floor),
                continuation,
            ),
            &where_,
        );
    }
    let floor_raises = run.floor_raises_in_the_mount_writes.expect("挂载做成时交回抬 F 的界");
    let mut second_base = crash.clone();
    let mut applied = 0usize;
    for (segment_index, segment) in mount_segments.iter().enumerate() {
        let first_write = segment[0];
        second_base.apply_writes(&mount_writes[applied..first_write]);
        applied = first_write;
        let length = segment.len();
        for mask in subsets(length, second_crash_cap, seed ^ (segment_index as u64) << 20 ^ state.segment as u64) {
            let persisted: Vec<bool> = (0..length).map(|bit| mask >> bit & 1 == 1).collect();
            let image = CrashImage {
                base: &second_base,
                writes: &mount_writes[first_write..first_write + length],
                persisted: persisted.clone(),
            };
            let second = read_recovery_fields(&image, ComparedVersion::TheRecoveredVersion);
            let mut persisted_in_the_mount = vec![false; mount_writes.len()];
            persisted_in_the_mount[..first_write].fill(true);
            persisted_in_the_mount[first_write..first_write + length].copy_from_slice(&persisted);
            tally.add(format!("second-crash/{tag}"));
            record_transparency(
                tally,
                &format!("{tag}/second"),
                &judge_mount_transparency(
                    &first,
                    &second,
                    MountTransparencyAction::SecondCrashInsideTheMount(
                        floor_raises.at_a_second_crash(&persisted_in_the_mount),
                    ),
                    continuation,
                ),
                &format!("{where_} 挂载段 {segment_index} mask {mask:#x}/{length}"),
            );
        }
    }
}
