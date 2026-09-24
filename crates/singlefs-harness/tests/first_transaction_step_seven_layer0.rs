//! 里程碑「第一个事务」步 7 的验收：拿步 5 的录制流按 D13（验证路线） 已定项 4 枚举全部崩溃状态，每个状态跑三件事——
//! 步 6 的恢复与 oracle、池级 checker（一元判决）、记录核对器（二元判决）。oracle 那一半的计数与 E142（第一个事务的干跑）
//! 第八次跑产物的 `name=layer0` / `name=journal_effect` 行逐字对；再加一组靶向的阳性对照。

mod common;

use common::{build_pool, file_content, geometry};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_harness::crash::{
    check_records, closed_form_state_count, enumerate_layer0_selecting,
    enumerate_layer0_selecting_versions_observing_each_state, evaluate_state, oracle_violation,
    root_identity_written_by, writes_and_segments, CrashImage, Layer0Tally, MemoryPool,
    PublishedVersion, RetainedWrite,
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

/// checker 与记录核对器那一半的钉死值。只在根槽 txg 3 已持久的状态上才走得到记账树、inode 树与分配记录树，那 14 条只在这些状态上评估
/// （I-5.4（分配记录罩住的槽互不相交） 在其中，
/// I-3.11（已分配减 defer 等于最新根走读） 与 I-3.1（已分配统计对得上） 同一个理由——要最新根下面的记账树：
/// 种子根与暖机根指着的第 0 版树表是空的，没有分配记录树；
/// I-1.10（码 2 条目宽等于字段表宽） 同一个理由：第 0 版树表一条条目都没有，没有哪个码 2 节点的条目宽可比）；
/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 单列一类：它除了回退候选集里那几版，还读下一次挂载会先施加的那一版
/// （`.claude/kb/invariants.md` I-3.10 那一行射程 ④），txg 3 的记录至少一份已落、根槽没落的状态上也评估，
/// 评估过的状态数由调用方按录制流逐状态现算（[`TransactionPublishInRecordedStream::allocation_generation_read_set_in`]），不写死；
/// I-9.14（树表条目的诞生 txg 跨根不变）在这条流上一个状态都评估不到；
/// I-8.6（反向链算法） 要环里至少有一条自证过的记录，环还空着的那几个状态上报「不适用」；
/// I-8.7（实例内事务号不重号） 与 I-8.8（前缀里的事务不被切开） 在这条流上一个状态都评估不到；其余 22 条每个状态都评估。
/// 违例只许出现在 I-7.7（系统配置实例代号不低于根环）：C322（取号那一步的屏障怎么放没有条款） 未还，
/// 取号只落了一块盘的那 2 个状态里两份系统配置的实例代号不等。
fn assert_checker_counts(
    tally: &Layer0Tally,
    every_state: u64,
    root_persisted_states: u64,
    states_with_an_empty_journal_ring: u64,
    states_judging_allocation_generations: u64,
) {
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据在已定的持久顺序下恒 0"
    );
    let only_under_the_new_root = [
        "I-1.10", "I-3.1", "I-3.9", "I-3.11", "I-5.2", "I-5.4", "I-9.1", "I-9.2", "I-9.4", "I-9.6",
        "I-9.7", "I-9.10", "I-9.12", "I-9.13",
    ];
    // I-3.10 除了回退候选集，还读下一次挂载会先施加的那一版（射程 ④）：新根没落而 txg 3 的记录已落时，
    // 恢复要由那条记录重建 txg 3 那一版，它的分配记录树在记录的新根段指着的树表下面。
    let under_the_new_root_or_through_the_record_the_next_mount_applies_first = ["I-3.10"];
    // I-9.14 要同一棵树的条目出现在两个树表单元里才比得出来，而这条流上只有第一个事务写树表：mkfs 种的第 0 版树表是空的、
    // 两次暖机空发布不写树表 ⇒ 每个状态都报「不适用」，一个状态都评估不到。跨根比得出来的流在
    // `second_transaction_step_zero_layer0.rs`（发布 B 起每次发布都重写树表），那里它真被评估过。
    // I-8.7 要同一个实例写出两条非 0 事务号的记录才比得出来，而这条流上实例 1 只发了一个承载事务的版本
    // （事务号 1）：写行与两次暖机都是事务号 0 的空发布（D23（journal 的角色与格式） 已定项 19 ①），不进那个序列
    // ⇒ 每个状态都报「不适用」，一个状态都评估不到。比得出来的流在 `second_transaction_step_zero_layer0.rs`
    // （发布 B 起同一实例接着往上数事务号），那里它进 `must_evaluate`、真被评估过。
    // I-8.8（前缀里的事务不被切开） 要同一个事务号落了两条以上的记录（③）、或者有一组一条提交标记都没有（④）才有对象，
    // 而第一版一事务一条、每条都带提交标记 ⇒ 每个状态都报「不适用」（提交标记字节照样判、判出别的值照样红，
    // 违例数仍由下面那条断言钉 0）。它在哪一条流上都评估不到，判别力在 `checker_known_bad_images.rs` 的阳性对照与坏镜像上。
    let never_comparable_on_this_stream = ["I-9.14", "I-8.7", "I-8.8"];
    // I-8.6 判的是「本实例内逻辑前一条」与这一条的链值，环里一条自证过的记录都没有时判不了：
    // 什么都没持久那一个状态，加上取号那一段（两盘各一次系统配置写）的 3 个非空子集，一共 4 个状态还没有记录落盘。
    // w1 的记录一落盘（两盘任一份）就判得了：它的计数器是 1 ⇒ 本实例第一条 ⇒ 反向链恒 0。
    let judged_once_a_record_is_on_disk = ["I-8.6"];
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        let expected_evaluated = if never_comparable_on_this_stream.contains(&invariant) {
            0
        } else if only_under_the_new_root.contains(&invariant) {
            root_persisted_states
        } else if under_the_new_root_or_through_the_record_the_next_mount_applies_first
            .contains(&invariant)
        {
            states_judging_allocation_generations
        } else if judged_once_a_record_is_on_disk.contains(&invariant) {
            every_state - states_with_an_empty_journal_ring
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
    transaction_publish: TransactionPublishInRecordedStream,
}

/// 这一状态上，承载第一个事务的那一版（txg 3）经哪条路进 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 的读集。
/// 这条流上只有它有分配记录树（种子根与暖机根指着的第 0 版树表是空的），它不进读集，I-3.10 在这个状态上就没有对象。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TransactionVersionInAllocationGenerationReadSet {
    /// 新根已落盘：它在回退候选集里。
    ThroughItsLandedRoot,
    /// 新根没落、这次发布的记录至少一份落了、盘上最新的根是它的前一版：下一次挂载会先施加这条记录（射程 ④）。
    ThroughTheRecordTheNextMountAppliesFirst,
    /// 新根没落，记录一份都没落、或盘上最新的根不是它的前一版。
    NotRead,
}

/// 录制流里承载第一个事务的那次发布：它的根槽写之前、上一条根槽写之后有单元写（两次暖机是零单元的空发布）。
/// I-3.10 那一类评估过的状态数按它逐状态现算：只看录制流里的写种类、写次序、根槽写里写下的根身份与这一状态里哪几次写持久了，
/// 不读盘、不经 checker 的代码（`.claude/kb/invariants.md`：判的一方不许与被判的一方共用同一段代码）。
struct TransactionPublishInRecordedStream {
    /// 它的根槽写（新根）。
    root_write_index: usize,
    /// 它的 journal 记录写：上一条根槽写之后、它的根槽写之前的那几条（两块盘各一份）。
    record_write_indexes: Vec<usize>,
    /// 它的前一版（同实例、checkpoint_txg 小 1）的根身份，按 (txg, 实例) 排：盘上最新的根是这一版时，
    /// 下一次挂载先施加的正是这次发布的记录。
    previous_root_identity: (CheckpointTxg, InstanceGeneration),
}

impl TransactionPublishInRecordedStream {
    fn find_in(writes: &[RetainedWrite]) -> Self {
        let mut transaction_publishes: Vec<(usize, Vec<usize>)> = Vec::new();
        let mut unit_writes_since_the_last_root = 0usize;
        let mut record_writes_since_the_last_root: Vec<usize> = Vec::new();
        for (write_index, write) in writes.iter().enumerate() {
            match write.kind {
                StepKind::UnitWrite => unit_writes_since_the_last_root += 1,
                StepKind::JournalRecord => record_writes_since_the_last_root.push(write_index),
                StepKind::RootRecordFua => {
                    let record_write_indexes =
                        std::mem::take(&mut record_writes_since_the_last_root);
                    if unit_writes_since_the_last_root > 0 {
                        transaction_publishes.push((write_index, record_write_indexes));
                    }
                    unit_writes_since_the_last_root = 0;
                }
                StepKind::ZeroFill | StepKind::SystemConfigurationSlot | StepKind::Barrier => {}
            }
        }
        assert_eq!(
            transaction_publishes.len(),
            1,
            "这条流上写单元的发布只有第一个事务那一次"
        );
        let (root_write_index, record_write_indexes) = transaction_publishes.remove(0);
        assert!(
            !record_write_indexes.is_empty(),
            "第一个事务那次发布写了 journal 记录"
        );
        let (checkpoint_txg, instance) = root_identity_of(&writes[root_write_index]);
        let previous_root_identity = (
            CheckpointTxg(
                checkpoint_txg
                    .0
                    .checked_sub(1)
                    .expect("第一个事务之前有两次暖机发布，它的 txg 不是 0"),
            ),
            instance,
        );
        assert!(
            writes
                .iter()
                .any(|write| write.kind == StepKind::RootRecordFua
                    && root_identity_of(write) == previous_root_identity),
            "前一版（同实例、txg 小 1）的根由这条流写出：基线里只有 mkfs 的种子根（实例 0、txg 0），比它旧"
        );
        Self {
            root_write_index,
            record_write_indexes,
            previous_root_identity,
        }
    }

    /// 层 0 的写要么整条落、要么整条不落：落了而没被更晚的已持久写盖过的那份记录，盘上字节就是录制流里写下的那一条，照原样自证。
    fn allocation_generation_read_set_in(
        &self,
        writes: &[RetainedWrite],
        persisted: &[bool],
    ) -> TransactionVersionInAllocationGenerationReadSet {
        if write_landed_and_not_overwritten(writes, persisted, self.root_write_index) {
            return TransactionVersionInAllocationGenerationReadSet::ThroughItsLandedRoot;
        }
        let some_record_copy_landed = self.record_write_indexes.iter().any(|record_write_index| {
            write_landed_and_not_overwritten(writes, persisted, *record_write_index)
        });
        if some_record_copy_landed
            && newest_landed_root_identity(writes, persisted) == Some(self.previous_root_identity)
        {
            TransactionVersionInAllocationGenerationReadSet::ThroughTheRecordTheNextMountAppliesFirst
        } else {
            TransactionVersionInAllocationGenerationReadSet::NotRead
        }
    }
}

/// 根槽写里写下的根身份，按 (txg, 实例) 排（D22（单元原子性怎么合成） 已定项 7 的择新序）。
fn root_identity_of(write: &RetainedWrite) -> (CheckpointTxg, InstanceGeneration) {
    let (instance, checkpoint_txg) =
        root_identity_written_by(write.bytes().expect("根槽写是普通写，带着字节"));
    (checkpoint_txg, instance)
}

/// 这次写持久了，而且之后没有哪次已持久的写落在同一块盘、与它重叠的字节上。
fn write_landed_and_not_overwritten(
    writes: &[RetainedWrite],
    persisted: &[bool],
    write_index: usize,
) -> bool {
    let write = &writes[write_index];
    let write_end = write.offset.0 + write.length_in_bytes();
    persisted[write_index]
        && !writes.iter().zip(persisted).skip(write_index + 1).any(
            |(later_write, later_is_persisted)| {
                *later_is_persisted
                    && later_write.device == write.device
                    && later_write.offset.0 < write_end
                    && write.offset.0 < later_write.offset.0 + later_write.length_in_bytes()
            },
        )
}

/// 这一状态里落了而没被盖过的根槽写中最新的那一条的身份；一条都没有时是 `None`（盘上只剩基线里 mkfs 的种子根）。
fn newest_landed_root_identity(
    writes: &[RetainedWrite],
    persisted: &[bool],
) -> Option<(CheckpointTxg, InstanceGeneration)> {
    writes
        .iter()
        .enumerate()
        .filter(|(write_index, write)| {
            write.kind == StepKind::RootRecordFua
                && write_landed_and_not_overwritten(writes, persisted, *write_index)
        })
        .map(|(_, write)| root_identity_of(write))
        .max()
}

/// I-3.10 那一类逐状态现算出来的计数。
#[derive(Debug, Default)]
struct AllocationGenerationReadSetCounts {
    states_through_the_landed_root: u64,
    states_through_the_record_the_next_mount_applies_first: u64,
}

impl AllocationGenerationReadSetCounts {
    fn count(&mut self, read_set: TransactionVersionInAllocationGenerationReadSet) {
        match read_set {
            TransactionVersionInAllocationGenerationReadSet::ThroughItsLandedRoot => {
                self.states_through_the_landed_root += 1;
            }
            TransactionVersionInAllocationGenerationReadSet::ThroughTheRecordTheNextMountAppliesFirst => {
                self.states_through_the_record_the_next_mount_applies_first += 1;
            }
            TransactionVersionInAllocationGenerationReadSet::NotRead => {}
        }
    }

    fn states_judging_allocation_generations(&self) -> u64 {
        self.states_through_the_landed_root
            + self.states_through_the_record_the_next_mount_applies_first
    }
}

/// 单版本形态的 oracle 那一份版本表：被判的那次根槽写出的那一代就是唯一带文件的版本（与 `enumerate_layer0_selecting` 里造的相同）。
fn single_version(prepared: &Prepared) -> Vec<PublishedVersion> {
    let (checkpoint_txg, instance) = root_identity_of(&prepared.writes[prepared.root_index]);
    vec![PublishedVersion {
        instance,
        checkpoint_txg,
        content: file_content(),
    }]
}

/// 按段枚举（`expand` 决定哪一段展开子集），每个状态上逐个现算 I-3.10 那一类。
fn enumerate_counting_allocation_generation_read_sets(
    prepared: &Prepared,
    expand: &dyn Fn(usize, &[usize]) -> bool,
) -> (Layer0Tally, AllocationGenerationReadSetCounts) {
    let versions = single_version(prepared);
    let mut read_set_counts = AllocationGenerationReadSetCounts::default();
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.root_index,
        &versions,
        expand,
        &mut |image, _consulted_report| {
            read_set_counts.count(
                prepared
                    .transaction_publish
                    .allocation_generation_read_set_in(image.writes, &image.persisted),
            );
        },
    );
    (tally, read_set_counts)
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
    let transaction_publish = TransactionPublishInRecordedStream::find_in(&writes);
    assert_eq!(
        transaction_publish.root_write_index, root_index,
        "被判的那条根槽写就是第一个事务那次发布的根"
    );
    Prepared {
        base,
        writes,
        segments,
        root_index,
        transaction_publish,
    }
}

/// 全量：262165 个状态。跑得慢（每个状态两遍恢复 + checker + 记录核对器），门禁 54 号在 release 下跑它；平时 `cargo test` 跳过。
#[test]
#[ignore = "全量 262165 个状态要一两分钟，门禁 54 号（.claude/gate.d/54-layer0-replay.sh）在 release 下跑"]
fn layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations() {
    let prepared = prepare("layer0-full");
    let closed_form = closed_form_state_count(&prepared.segments);
    let (tally, read_set_counts) = enumerate_counting_allocation_generation_read_sets(
        &prepared,
        &|_segment_index, _segment| true,
    );
    println!(
        "LAYER0 states={} closed_form={closed_form} violations={} root_persisted_states={} no_file={} file_read={} failed={} verification_ran={} verification_failed={} journal_differing={} exhaustive={} states_by_publish=[{}]",
        tally.states,
        tally.violations,
        tally.root_persisted_states,
        tally.no_file_states,
        tally.file_read_states,
        tally.failed_states,
        tally.verification_ran_states,
        tally.verification_failed_states,
        tally.journal_differing_states,
        tally.states == closed_form,
        tally.states_by_publish_text()
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
    // 按发布分（里程碑「第二个事务」步 6 验收第 1 条的口径，第一条流同样报）：暖机 txg 1、2 各 3 + 3 + 1，
    // A 是 18 写段 262143 + 记录段 3 + 根槽段 1，A 的系统配置槽轮换 3，再加全部持久那一个。
    assert_eq!(
        tally.states_by_publish_text(),
        "instance1_txg1=7 instance1_txg2=7 instance1_txg3=262147 after_the_last_root=3 every_write_persisted=1"
    );
    assert_allocation_generation_read_set_reached_through_the_record(&read_set_counts);
    assert_checker_counts(
        &tally,
        262_165,
        4,
        4,
        read_set_counts.states_judging_allocation_generations(),
    );
}

/// I-3.10 那一类的判别力：枚举里要有「新根没落、记录已落」的状态，否则射程 ④ 那一步撤回了这里也看不出来。
fn assert_allocation_generation_read_set_reached_through_the_record(
    read_set_counts: &AllocationGenerationReadSetCounts,
) {
    assert!(
        read_set_counts.states_through_the_record_the_next_mount_applies_first > 0,
        "枚举里没有「txg 3 的记录已落、根槽没落」的状态：I-3.10 读下一次挂载会先施加的那一版这一步在这里判不出来（{read_set_counts:?}）"
    );
}

/// 平时跑的那一份：18 个写的那一段不展开子集（只作为整段持久），其余九段全展开——22 个状态；报出来的是「不是全量」。
#[test]
fn layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_full_tally_shape() {
    let prepared = prepare("layer0-partial");
    let (tally, read_set_counts) = enumerate_counting_allocation_generation_read_sets(
        &prepared,
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
    assert_eq!(
        tally.states_by_publish_text(),
        "instance1_txg1=7 instance1_txg2=7 instance1_txg3=4 after_the_last_root=3 every_write_persisted=1",
        "22 个状态按发布分：少的只在 A 那一格（18 写段不展开）"
    );
    assert_allocation_generation_read_set_reached_through_the_record(&read_set_counts);
    assert_checker_counts(
        &tally,
        22,
        4,
        4,
        read_set_counts.states_judging_allocation_generations(),
    );
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
