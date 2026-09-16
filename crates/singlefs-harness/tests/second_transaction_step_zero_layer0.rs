//! 里程碑「第二个事务」步 0 在发布 B 上的那一半：把「取号 → 暖机 → A → B」整条录制流按 D13（验证路线） 已定项 4 枚举全部崩溃状态，
//! 与第一个事务同一种切法（上一次发布的超级块槽写与下一次发布的单元写落在同一段，登记表八末尾那条 ⚠️），
//! 每个状态跑恢复 + 多版本 oracle（实际走的根是哪一代就得读出那一代的内容）、池级 checker、记录核对器。
//! 平时 `cargo test` 跳过两个 18 写的段；全量那条标 ignored，54 号门禁在 release 下跑它。再加四组靶向的阳性对照。

mod common;

use common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::recovery::RecoveryOutcome;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::crash::{
    closed_form_state_count, enumerate_layer0_selecting_versions, enumerate_layer0_versions,
    evaluate_state_for_versions, writes_and_segments, Layer0Tally, MemoryPool, PublishedVersion,
    RetainedWrite,
};
use singlefs_harness::segments::StepKind;

const SECOND_FILE_BYTES: usize = 4100;

fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

struct Prepared {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    /// B 的根槽 FUA 写在写表里的下标。
    judged_root_index: usize,
    /// A 的根槽 FUA 写在写表里的下标：它之后的写都是 B 的。
    first_root_index: usize,
    versions: Vec<PublishedVersion>,
}

fn overwrite(pool: &mut BuiltPool) {
    let parameters = parameters();
    let content = second_content();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &pool.output,
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        InstanceGeneration(1),
    )
    .expect("覆盖写");
}

fn prepare(tag: &str) -> Prepared {
    let mut pool = build_pool(tag);
    overwrite(&mut pool);
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 2],
        "A 的两个超级块槽写与 B 的 16 个单元写合成一段：整条流按屏障切，不按发布切"
    );
    assert_eq!(writes.len(), 54, "取号 2 + 暖机 10 + A 21 + B 21 次写");
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(root_indexes.len(), 4, "暖机两代 + A + B 四条根槽写");
    Prepared {
        base,
        writes,
        segments,
        judged_root_index: root_indexes[3],
        first_root_index: root_indexes[2],
        versions: vec![
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(3),
                content: file_content(),
            },
            PublishedVersion {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
                content: second_content(),
            },
        ],
    }
}

fn assert_checker_and_record_checker_clean(tally: &Layer0Tally) {
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据在已定的持久顺序下恒 0"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant} 在两次发布的流上判违例的状态数必须是 0"
        );
    }
    for must_evaluate in ["I-3.1", "I-5.2", "I-5.1", "I-7.2"] {
        assert!(
            tally
                .checker_evaluated_states
                .get(must_evaluate)
                .copied()
                .unwrap_or(0)
                > 0,
            "{must_evaluate} 至少在一个状态上真被评估过"
        );
    }
}

/// 平时跑的那一份：两个 18 写的段不展开（只以整段持久进入后面的状态），其余每段任意子集。
#[test]
fn every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims() {
    let prepared = prepare("layer0-b-fast");
    let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 18;
    let tally = enumerate_layer0_selecting_versions(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        &expand,
    );
    let expanded: Vec<Vec<usize>> = prepared
        .segments
        .iter()
        .filter(|segment| segment.len() < 18)
        .cloned()
        .collect();
    assert_eq!(
        tally.states,
        closed_form_state_count(&expanded),
        "展开的段按闭式数"
    );
    assert_eq!(tally.states, 26);
    assert_eq!(
        tally.violations, 0,
        "第一处违例：{:?}",
        tally.first_violation
    );
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert!(tally.file_read_states > 0 && tally.no_file_states > 0);
    assert_checker_and_record_checker_clean(&tally);
}

/// 全量：两次发布的流 13 段、闭式 524312 个状态（第一个事务的 262165 减掉末尾那个 2 写的段、加上 B 的四段与合成的 18 写段）。
/// 54 号门禁在 release 下跑它，认下面打印的 `LAYER0B` 行里 `exhaustive=true`。
#[test]
#[ignore = "全量 524312 个状态、每个两遍恢复 + checker，debug 下几分钟；门禁 54 号在 release 下跑"]
fn full_enumeration_of_the_two_publish_stream_is_exhaustive_and_clean() {
    let prepared = prepare("layer0-b-full");
    let closed_form = closed_form_state_count(&prepared.segments);
    assert_eq!(closed_form, 524_312, "闭式：1 + Σ(2^|段| − 1)，十三段");
    let tally = enumerate_layer0_versions(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
    );
    let checker_violations: u64 = tally.checker_violated_states.values().sum();
    println!(
        "LAYER0B states={} closed_form={closed_form} exhaustive={} violations={} root_persisted_states={} no_file={} file_read={} failed={} journal_differing={} verification_ran={} verification_failed={} record_root_without_record={} record_claimed_state_missing_unit={} checker_violations={checker_violations} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.root_persisted_states,
        tally.no_file_states,
        tally.file_read_states,
        tally.failed_states,
        tally.journal_differing_states,
        tally.verification_ran_states,
        tally.verification_failed_states,
        tally.record_root_without_record,
        tally.record_claimed_state_missing_unit,
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_eq!(tally.states, closed_form, "枚举到的状态数要等于闭式");
    assert_eq!(
        tally.violations, 0,
        "第一处违例：{:?}",
        tally.first_violation
    );
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(tally.failed_states, 0);
    assert_checker_and_record_checker_clean(&tally);
}

/// 靶向的阳性对照：把持久集合手工摆成四个形状，oracle、journal 承重、记录核对器各要在它该红的那一格红。
#[test]
fn targeted_controls_on_the_second_publish_go_red_where_they_should() {
    let prepared = prepare("layer0-b-controls");
    let all = vec![true; prepared.writes.len()];
    let is_second_publish = |index: usize| index > prepared.first_root_index;
    let kinds_of_second_publish = |kind: StepKind| -> Vec<usize> {
        prepared
            .writes
            .iter()
            .enumerate()
            .filter(|(index, write)| is_second_publish(*index) && write.kind == kind)
            .map(|(index, _)| index)
            .collect()
    };
    let second_publish_units = kinds_of_second_publish(StepKind::UnitWrite);
    let second_publish_records = kinds_of_second_publish(StepKind::JournalRecord);
    assert_eq!(
        (second_publish_units.len(), second_publish_records.len()),
        (16, 2)
    );
    let evaluate = |persisted: Vec<bool>| {
        let mut tally = Layer0Tally::default();
        let report = evaluate_state_for_versions(
            &prepared.base,
            &prepared.writes,
            persisted,
            prepared.judged_root_index,
            &prepared.versions,
            &mut tally,
        );
        (report, tally)
    };

    // ① 全部持久：走 B 的根、读回第二次的内容，零违例。
    let (all_persisted_report, all_persisted_tally) = evaluate(all.clone());
    assert_eq!(
        all_persisted_report.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(4)))
    );
    assert!(matches!(
        all_persisted_report.outcome,
        RecoveryOutcome::FileRead { .. }
    ));
    assert_eq!(all_persisted_tally.violations, 0);

    // ② B 的根槽已持久、B 的十六个单元写一份都没持久：oracle 必须红（走读失败），记录核对器判「自称新态而单元缺席」。
    let mut root_without_units = all.clone();
    for index in &second_publish_units {
        root_without_units[*index] = false;
    }
    let (_, root_without_units_tally) = evaluate(root_without_units);
    assert_eq!(
        root_without_units_tally.violations, 1,
        "根槽已持久而单元不在：{:?}",
        root_without_units_tally.first_violation
    );
    assert_eq!(
        root_without_units_tally.record_claimed_state_missing_unit,
        1
    );
    assert_eq!(
        root_without_units_tally.ignored_violations, 1,
        "不看 journal 那一遍同样走读失败"
    );

    // ③ B 的根槽没持久、记录与单元都持久：看 journal 由记录重建第 4 代根读出第二次的内容，不看就退回 A——journal 在这一格承重。
    let mut root_missing = all.clone();
    root_missing[prepared.judged_root_index] = false;
    let (root_missing_report, root_missing_tally) = evaluate(root_missing);
    assert_eq!(
        root_missing_report.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(4)))
    );
    assert_eq!(root_missing_report.journal.prefix_applied, 1);
    assert_eq!(root_missing_tally.journal_differing_states, 1);
    assert_eq!(root_missing_tally.violations, 0);

    // ④ B 的根槽已持久、两份记录都没持久：走读没问题（根记录自带全部字段）、oracle 不红，但记录核对器判「根在案而记录缺席」。
    let mut root_without_records = all.clone();
    for index in &second_publish_records {
        root_without_records[*index] = false;
    }
    let (_, root_without_records_tally) = evaluate(root_without_records);
    assert_eq!(root_without_records_tally.violations, 0);
    assert_eq!(root_without_records_tally.record_root_without_record, 1);

    // ⑤ B 一个字节都没持久：走 A 的根、读回第一次的内容，零违例——旧态在它自己的根下面是合法的。
    let mut nothing_of_second_publish = all;
    for (write_index, persisted) in nothing_of_second_publish.iter_mut().enumerate() {
        if is_second_publish(write_index) {
            *persisted = false;
        }
    }
    let (nothing_of_second_publish_report, nothing_of_second_publish_tally) =
        evaluate(nothing_of_second_publish);
    assert_eq!(
        nothing_of_second_publish_report.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(3)))
    );
    assert_eq!(
        nothing_of_second_publish_report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content: file_content()
        }
    );
    assert_eq!(nothing_of_second_publish_tally.violations, 0);
}
