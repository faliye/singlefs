//! 里程碑「第一个事务」步 7 的验收：拿步 5 的录制流按 D13（验证路线） 已定项 4 枚举全部崩溃状态，每个状态跑三件事——
//! 步 6 的恢复与 oracle、池级 checker（一元判决）、记录核对器（二元判决）。oracle 那一半的计数与 E142（第一个事务的干跑）
//! 第八次跑产物的 `name=layer0` / `name=journal_effect` 行逐字对；再加一组靶向的阳性对照。

mod common;

use common::{build_pool, file_content, geometry};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_harness::crash::{
    check_records, closed_form_state_count, enumerate_layer0, enumerate_layer0_selecting,
    evaluate_state, oracle_violation, writes_and_segments, CrashImage, Layer0Tally, MemoryPool,
    RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::RecordedOperationKind;

/// oracle 那一半：产物第 45 行逐字 `states=262165 … violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6
/// verification_failed=0 first_violation=none`，第 48 行 `differing_states=3`。
#[derive(Debug, PartialEq, Eq)]
struct OracleCounts {
    states: u64,
    violations: u64,
    root_persisted_states: u64,
    no_file_states: u64,
    file_read_states: u64,
    failed_states: u64,
    journal_differing_states: u64,
    verification_ran_states: u64,
    verification_failed_states: u64,
    first_violation: Option<String>,
}

fn oracle_counts(tally: &Layer0Tally) -> OracleCounts {
    OracleCounts {
        states: tally.states,
        violations: tally.violations,
        root_persisted_states: tally.root_persisted_states,
        no_file_states: tally.no_file_states,
        file_read_states: tally.file_read_states,
        failed_states: tally.failed_states,
        journal_differing_states: tally.journal_differing_states,
        verification_ran_states: tally.verification_ran_states,
        verification_failed_states: tally.verification_failed_states,
        first_violation: tally.first_violation.clone(),
    }
}

/// checker 与记录核对器那一半打成一行：每条不变量「评估过的状态数 / 判违例的状态数」。
fn checker_line(tally: &Layer0Tally) -> String {
    let per_invariant: Vec<String> = singlefs_checker::image::IMPLEMENTED_INVARIANTS
        .iter()
        .map(|invariant| {
            format!(
                "{invariant}={}/{}",
                tally
                    .checker_evaluated_states
                    .get(invariant)
                    .copied()
                    .unwrap_or(0),
                tally
                    .checker_violated_states
                    .get(invariant)
                    .copied()
                    .unwrap_or(0)
            )
        })
        .collect();
    format!(
        "CHECKER record_root_without_record={} record_claimed_state_missing_unit={} {}",
        tally.record_root_without_record,
        tally.record_claimed_state_missing_unit,
        per_invariant.join(" ")
    )
}

/// checker 与记录核对器那一半的钉死值。只在根槽 txg 3 已持久的状态上才走得到记账树、inode 树与分配记录树，那 10 条只在这些状态上评估
/// （I-5.4（分配记录罩住的槽互不相交） 在其中：种子根与暖机根指着的第 0 版树表是空的，没有分配记录树）；
/// I-9.14（树表条目的诞生 txg 跨根不变）在这条流上一个状态都评估不到；其余 18 条每个状态都评估。
/// 违例只许出现在 I-7.7（超级块实例代号不低于根环）：C322（取号那一步的屏障怎么放没有条款） 未还，
/// 取号只落了一块盘的那 2 个状态里两份超级块的实例代号不等。
fn assert_checker_counts(tally: &Layer0Tally, every_state: u64, root_persisted_states: u64) {
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据在已定的持久顺序下恒 0"
    );
    let only_under_the_new_root = [
        "I-3.1", "I-3.9", "I-5.2", "I-5.4", "I-9.1", "I-9.2", "I-9.4", "I-9.7", "I-9.10", "I-9.13",
    ];
    // I-9.14 要同一棵树的条目出现在两个树表单元里才比得出来，而这条流上只有第一个事务写树表：mkfs 种的第 0 版树表是空的、
    // 两次暖机空发布不写树表 ⇒ 每个状态都报「不适用」，一个状态都评估不到。跨根比得出来的流在
    // `second_transaction_step_zero_layer0.rs`（发布 B 起每次发布都重写树表），那里它真被评估过。
    let never_comparable_on_this_stream = ["I-9.14"];
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        let expected_evaluated = if never_comparable_on_this_stream.contains(&invariant) {
            0
        } else if only_under_the_new_root.contains(&invariant) {
            root_persisted_states
        } else {
            every_state
        };
        assert_eq!(
            tally
                .checker_evaluated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            expected_evaluated,
            "{invariant} 评估过的状态数（阴性结果要能和「代码没跑到」分开）"
        );
        assert_eq!(
            tally.checker_violated_states.get(invariant).copied().unwrap_or(0),
            0,
            "{invariant} 判违例的状态数必须是 0（取号只落了一块盘的 2 个状态按 2026-09-14 定案的 I-7.7 ① ② 判成立，C322（取号那一步的屏障怎么放没有条款） 还清）"
        );
    }
}

struct Prepared {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    root_index: usize,
}

fn prepare(tag: &str) -> Prepared {
    let pool = build_pool(tag);
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 2]
    );
    assert_eq!(writes.len(), 33, "mkfs 之后 39 步里 33 次写、6 道屏障");
    let root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("写流里有根槽那一条");
    Prepared {
        base,
        writes,
        segments,
        root_index,
    }
}

/// 全量：262165 个状态。跑得慢（每个状态两遍恢复 + checker + 记录核对器），门禁 54 号在 release 下跑它；平时 `cargo test` 跳过。
#[test]
#[ignore = "全量 262165 个状态要一两分钟，门禁 54 号（.claude/gate.d/54-layer0-replay.sh）在 release 下跑"]
fn layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations() {
    let prepared = prepare("layer0-full");
    let closed_form = closed_form_state_count(&prepared.segments);
    let tally = enumerate_layer0(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.root_index,
        &file_content(),
    );
    println!(
        "LAYER0 states={} closed_form={closed_form} violations={} root_persisted_states={} no_file={} file_read={} failed={} verification_ran={} verification_failed={} journal_differing={} exhaustive={}",
        tally.states,
        tally.violations,
        tally.root_persisted_states,
        tally.no_file_states,
        tally.file_read_states,
        tally.failed_states,
        tally.verification_ran_states,
        tally.verification_failed_states,
        tally.journal_differing_states,
        tally.states == closed_form
    );
    println!("{}", checker_line(&tally));
    println!("CHECKER_FIRST {:?}", tally.checker_first_violation);
    assert_eq!(
        tally.ignored_violations, 0,
        "不看 journal 那一遍恢复同样过 oracle：{:?}",
        tally.first_ignored_violation
    );
    assert_eq!(
        oracle_counts(&tally),
        OracleCounts {
            states: 262_165,
            violations: 0,
            root_persisted_states: 4,
            no_file_states: 262_158,
            file_read_states: 7,
            failed_states: 0,
            journal_differing_states: 3,
            verification_ran_states: 6,
            verification_failed_states: 0,
            first_violation: None,
        }
    );
    assert_eq!(closed_form, 262_165, "全量：枚举到的状态数等于闭式");
    assert_checker_counts(&tally, 262_165, 4);
}

/// 平时跑的那一份：18 个写的那一段不展开子集（只作为整段持久），其余九段全展开——22 个状态；报出来的是「不是全量」。
#[test]
fn layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_full_tally_shape() {
    let prepared = prepare("layer0-partial");
    let tally = enumerate_layer0_selecting(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.root_index,
        &file_content(),
        &|_segment_index, segment| segment.len() < 18,
    );
    println!("{}", checker_line(&tally));
    println!("CHECKER_FIRST {:?}", tally.checker_first_violation);
    assert_eq!(
        oracle_counts(&tally),
        OracleCounts {
            states: 22,
            violations: 0,
            root_persisted_states: 4,
            no_file_states: 15,
            file_read_states: 7,
            failed_states: 0,
            journal_differing_states: 3,
            verification_ran_states: 6,
            verification_failed_states: 0,
            first_violation: None,
        },
        "少展开的只有 18 个写那一段的 262143 个子集：文件在 / 根已持久 / 验证跑过 / journal 承重的状态数都与全量相同"
    );
    assert_checker_counts(&tally, 22, 4);
    assert_eq!(closed_form_state_count(&prepared.segments), 262_165);
}

/// checker 在写完第一个事务的镜像上：每条第一版不变量的判定。
#[test]
fn checker_report_on_the_final_image() {
    let pool = build_pool("checker-final");
    for (invariant, verdict) in check_pool_image(&pool.memory_pool()) {
        println!("FINAL {invariant} {verdict:?}");
    }
}

/// 阳性对照（靶向，不是 E142 那条单盘无屏障臂）：根槽已持久而某个单元两份都没持久——这正是段与段之间少一道屏障会放进来的那类状态。
/// oracle 对 8 个单元逐个都要判红；全部持久那一个判绿；根槽已持久而 journal 记录两份都没持久不算违例（根自己带着全部字段）。
#[test]
fn positive_control_root_persisted_without_each_unit_is_caught_by_the_oracle() {
    let prepared = prepare("layer0-control");
    let all_persisted = vec![true; prepared.writes.len()];
    let mut tally = Layer0Tally::default();
    let report = evaluate_state(
        &prepared.base,
        &prepared.writes,
        all_persisted.clone(),
        prepared.root_index,
        &file_content(),
        &mut tally,
    );
    assert_eq!(
        oracle_violation(&report.outcome, true, &file_content()),
        None
    );
    let unit_write_indices: Vec<usize> = prepared
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::UnitWrite)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(unit_write_indices.len(), 16);
    let mut violations = 0;
    for pair in unit_write_indices.chunks(2) {
        let mut persisted = all_persisted.clone();
        for index in pair {
            persisted[*index] = false;
        }
        let mut control_tally = Layer0Tally::default();
        let control_report = evaluate_state(
            &prepared.base,
            &prepared.writes,
            persisted,
            prepared.root_index,
            &file_content(),
            &mut control_tally,
        );
        if oracle_violation(&control_report.outcome, true, &file_content()).is_some() {
            violations += 1;
        }
        assert_eq!(
            control_tally.violations, 1,
            "根槽已持久而单元 {pair:?} 两份都没持久：必须判红"
        );
    }
    assert_eq!(violations, 8, "8 个单元逐个抽掉，8 次都判红");
    let journal_indices: Vec<usize> = prepared
        .writes
        .iter()
        .enumerate()
        .filter(|(_, write)| {
            write.kind == StepKind::JournalRecord
                && write.offset.0
                    == singlefs_core::journal::record_offset(
                        3,
                        singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
                    )
                    .0
        })
        .map(|(index, _)| index)
        .collect();
    assert_eq!(journal_indices.len(), 2);
    let mut persisted = all_persisted;
    for index in journal_indices {
        persisted[index] = false;
    }
    let mut control_tally = Layer0Tally::default();
    evaluate_state(
        &prepared.base,
        &prepared.writes,
        persisted,
        prepared.root_index,
        &file_content(),
        &mut control_tally,
    );
    assert_eq!(control_tally.violations, 0);
    assert_eq!(
        control_tally.record_root_without_record, 1,
        "根槽已持久而记录两份都没持久：oracle 不红，记录核对器红"
    );
}

/// 记录核对器的判别力（C6（块层语义假设写错） 那条「崩溃测试必须变红」在模型层的形态，与 E77（发布的持久顺序） b_ur 臂同形）：
/// 摘掉第一个事务里「journal 记录 → 根槽」那道屏障，两份记录与根槽同段；枚举里出现根在案而记录一份都不在的状态。oracle 在这里不红。
#[test]
fn removing_the_barrier_before_the_root_slot_is_caught_by_the_record_checker() {
    let pool = build_pool("record-checker-barrier-two");
    let base = pool.memory_pool_after_mkfs();
    let mut operations = pool.retained_operations()[pool.mkfs_operation_count..].to_vec();
    let root_position = operations
        .iter()
        .rposition(|retained| geometry().classify(&retained.operation) == StepKind::RootRecordFua)
        .expect("有根槽写");
    let barrier_position = operations[..root_position]
        .iter()
        .rposition(|retained| retained.operation.kind == RecordedOperationKind::Barrier)
        .expect("根槽之前有屏障");
    operations.remove(barrier_position);
    let (writes, segments) = writes_and_segments(&operations, &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 2, 1, 2, 2, 1, 18, 3, 2],
        "记录两份与根槽并成一段"
    );
    let root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽");
    let tally = enumerate_layer0_selecting(
        &base,
        &writes,
        &segments,
        root_index,
        &file_content(),
        &|_segment_index, segment| segment.len() < 18,
    );
    assert_eq!(tally.violations, 0, "oracle 不红：根记录自己带着全部字段");
    assert_eq!(
        tally.record_root_without_record, 1,
        "三个写那一段的真子集里只有「只有根槽」这一个是根在案而两份记录都不在"
    );
    assert_eq!(tally.record_claimed_state_missing_unit, 0);
}

/// 记录核对器的另一条判据：「单元 → journal 记录」那道屏障不在时，会出现两份记录在、某个单元两份都不在、根槽没在的状态。
/// 不验点名单元的恢复（只供测试强制进入的 `ConsultWithoutNamedVerification`）在这个状态上施加那条记录、自称到了 txg 3，
/// 记录核对器判「恢复自称新态而单元缺席」；验点名单元的恢复停在 txg 2，不判。
#[test]
fn applying_a_record_whose_units_are_missing_is_caught_by_the_record_checker() {
    let pool = build_pool("record-checker-barrier-one");
    let base = pool.memory_pool_after_mkfs();
    let (writes, _) = writes_and_segments(
        &pool.retained_operations()[pool.mkfs_operation_count..],
        &geometry(),
    );
    let root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽");
    let first_unit = writes
        .iter()
        .position(|write| write.kind == StepKind::UnitWrite)
        .expect("第一个单元");
    assert_eq!(
        writes[first_unit].offset,
        writes[first_unit + 1].offset,
        "同一个单元的两份紧挨着写"
    );
    let mut persisted = vec![false; writes.len()];
    for flag in persisted.iter_mut().take(root_index) {
        *flag = true;
    }
    persisted[first_unit] = false;
    persisted[first_unit + 1] = false;
    let image = CrashImage {
        base: &base,
        writes: &writes,
        persisted,
    };
    let naive = recover(&image, JournalPolicy::ConsultWithoutNamedVerification);
    assert_eq!(
        naive.effective_root.map(|(_, txg)| txg.0),
        Some(3),
        "不验就施加了记录 3"
    );
    assert!(
        check_records(&image, naive.effective_root).claimed_state_missing_unit,
        "自称到了 txg 3 而 t1 两份都不在"
    );
    let validating = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        validating.effective_root.map(|(_, txg)| txg.0),
        Some(2),
        "验点名单元的恢复停在 txg 2"
    );
    assert!(!check_records(&image, validating.effective_root).claimed_state_missing_unit);
    assert!(!check_records(&image, validating.effective_root).root_without_record);
}
