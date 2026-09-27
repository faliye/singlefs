//! 里程碑「第一个事务」步 7 的验收：拿步 5 的录制流按 D13（验证路线） 已定项 4 枚举全部崩溃状态，每个状态跑三件事——
//! 步 6 的恢复与 oracle、池级 checker（一元判决）、记录核对器（二元判决）。oracle 那一半的计数与 E142（第一个事务的干跑）
//! 第八次跑产物的 `name=layer0` / `name=journal_effect` 行逐字对；再加一组靶向的阳性对照。

mod common;

use common::{build_pool, file_content, geometry};
use singlefs_checker::image::MAPPING_KEY_MATCHES_THE_UNIT_HEADER;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::recovery::RecoveryReport;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_harness::crash::{
    check_records, closed_form_state_count, enumerate_layer0_in_state_slices_or_one_shard,
    enumerate_layer0_selecting, evaluate_state, full_expansion, layer0_state_count,
    layer0_state_count_with_torn_in_place_overwrites, oracle_violation, quick_tier_expansion,
    root_identity_written_by, writes_and_segments, CrashImage, Layer0EnumerationOutcome,
    Layer0ObserverCounts, Layer0Parallelism, Layer0SegmentExpansion, Layer0Tally, MemoryPool,
    PublishedVersion, RetainedWrite,
};
use singlefs_harness::layer0_progress::Layer0Resume;
use singlefs_harness::segments::StepKind;
use singlefs_harness::RecordedOperationKind;

/// oracle 那一半：字段照 E142（第一个事务的干跑） 产物的 `name=layer0` / `name=journal_effect` 两行（第八次跑那一份是 A 只有 16 个单元写时的形状：
/// `states=262165 … no_file=262158 file_read=7 … verification_ran=6`、`differing_states=3`；今天 A 有 24 个单元写，数由下面的用例钉）。
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

/// checker 与记录核对器那一半的钉死值。只在根槽 txg 3 已持久的状态上才走得到记账树、inode 树、分配记录树与中央映射树的条目，
/// 那 16 条只在这些状态上评估
/// （I-5.4（分配记录罩住的槽互不相交） 在其中，
/// 映射 key 与单元头相符（`MAPPING_KEY_MATCHES_THE_UNIT_HEADER`，实审 B2 新立、编号待定）判的是中央映射树叶里的每一条条目：
/// 种子根与暖机根的映射根指针全零（没有文件的一版），走不到条目；txg 3 的记录已落而根槽没落时，由记录施加出来的那一版
/// 在这条流上不走（它要实例表里有一行记着某次恢复施加到过 txg 3，实例表的行是挂载写行时写的，而这条流上唯一一次挂载在 txg 3 之前），
/// I-3.10 读下一次挂载会先施加的那一版只读树表、不走映射树，
/// I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉） 与 I-9.7 同一个理由——要走得到 inode 叶容器里的记录，
/// I-3.11（已分配减 defer 等于最新根走读） 与 I-3.1（已分配统计对得上） 同一个理由——要最新根下面的记账树：
/// 种子根与暖机根指着的第 0 版树表是空的，没有分配记录树；
/// I-1.10（码 2 条目宽等于字段表宽） 同一个理由：第 0 版树表一条条目都没有，没有哪个码 2 节点的条目宽可比）；
/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 单列一类：它除了回退候选集里那几版，还读下一次挂载会先施加的那一版
/// （`.claude/kb/invariants.md` I-3.10 那一行射程 ④），txg 3 的记录至少一份已落、根槽没落的状态上也评估，
/// 评估过的状态数由调用方按录制流逐状态现算（[`TransactionPublishInRecordedStream::allocation_generation_read_set_in`]），不写死；
/// I-9.14（树表条目的诞生 txg 跨根不变）在这条流上一个状态都评估不到；
/// I-8.6（反向链算法） 要环里至少有一条自证过的记录，环还空着的那几个状态上报「不适用」；
/// I-8.7（实例内事务号不重号）、I-8.8（前缀里的事务不被切开） 与 I-7.9（回退下界 F 不高于抬 F 的上限） 在这条流上一个状态都评估不到；
/// 其余 23 条每个状态都评估。
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
        "I-1.10",
        "I-3.1",
        "I-3.9",
        "I-3.11",
        "I-5.2",
        "I-5.4",
        "I-9.1",
        "I-9.2",
        "I-9.4",
        "I-9.6",
        "I-9.7",
        "I-9.10",
        "I-9.12",
        "I-9.13",
        "I-9.15",
        MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
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
    // I-7.9（回退下界 F 不高于抬 F 的上限） 只判抬 F 的那一条根，这条流上没抬过 F（每条根带的 F 都是 0）⇒ 每个状态都报「不适用」。
    // 抬过 F 的流在 `second_transaction_step_zero_layer0.rs`（E 之前抬到 11），那里它真被评估过。
    let never_comparable_on_this_stream = ["I-9.14", "I-8.7", "I-8.8", "I-7.9"];
    // I-8.6 判的是「本实例内逻辑前一条」与这一条的链值，环里一条自证过的记录都没有时判不了：
    // 取号那一段（两盘各一次系统配置写，原地覆写各三态）的 3² − 1 个状态，加上暖机第一次的记录段整段没落那一个，
    // 一共 9 个状态还没有记录落盘（补第三态之前 4 个）。
    // w1 的记录一落盘（两盘任一份）就判得了：它的计数器是 1 ⇒ 本实例第一条 ⇒ 反向链恒 0。
    // I-8.9（一次发布的记录序号连续且只有末条带标志） 同一个理由：环里有一条自证过的记录就判得了（记录标志其余位、序号 0 逐条判），
    // 没有就报不适用；这条流上 w1、w4、t9 各是一次只有一条记录的发布（序号 1、带末条标志）。
    let judged_once_a_record_is_on_disk = ["I-8.6", "I-8.9"];
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

/// 观察者计数里这两类的名字（随进度文件续跑，`crash::Layer0ObserverCounts`）。
const STATES_THROUGH_THE_LANDED_ROOT: &str = "states_through_the_landed_root";
const STATES_THROUGH_THE_RECORD_THE_NEXT_MOUNT_APPLIES_FIRST: &str =
    "states_through_the_record_the_next_mount_applies_first";

impl AllocationGenerationReadSetCounts {
    /// 一个状态记进观察者计数：续跑时读回的片不再给观察者看，这两类的累计要随片进进度文件。
    fn count_into(
        read_set: TransactionVersionInAllocationGenerationReadSet,
        counts: &mut Layer0ObserverCounts,
    ) {
        match read_set {
            TransactionVersionInAllocationGenerationReadSet::ThroughItsLandedRoot => {
                counts.add(STATES_THROUGH_THE_LANDED_ROOT, 1);
            }
            TransactionVersionInAllocationGenerationReadSet::ThroughTheRecordTheNextMountAppliesFirst => {
                counts.add(STATES_THROUGH_THE_RECORD_THE_NEXT_MOUNT_APPLIES_FIRST, 1);
            }
            TransactionVersionInAllocationGenerationReadSet::NotRead => {}
        }
    }

    fn from_observer_counts(counts: &Layer0ObserverCounts) -> Self {
        Self {
            states_through_the_landed_root: counts.get(STATES_THROUGH_THE_LANDED_ROOT),
            states_through_the_record_the_next_mount_applies_first: counts
                .get(STATES_THROUGH_THE_RECORD_THE_NEXT_MOUNT_APPLIES_FIRST),
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

/// 按段枚举（`expansion` 决定每一段怎么展开），每个状态上逐个现算 I-3.10 那一类，记进观察者计数（续跑时随进度文件接上；
/// 双机分片时随账本到 merge 那一趟）。分片跑一片（`SINGLEFS_LAYER0_SHARD=<i>/<n>`）时交回 `None`：这一片的计数进了账本，整条流的等 merge。
fn enumerate_counting_allocation_generation_read_sets(
    prepared: &Prepared,
    expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
    resume: &Layer0Resume,
) -> Option<(Layer0Tally, AllocationGenerationReadSetCounts)> {
    let versions = single_version(prepared);
    let outcome = enumerate_layer0_in_state_slices_or_one_shard(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.root_index,
        &versions,
        expansion,
        Layer0Parallelism::from_environment(),
        Some(&mut |image: &CrashImage<'_>,
                   _consulted_report: &RecoveryReport,
                   counts: &mut Layer0ObserverCounts| {
            AllocationGenerationReadSetCounts::count_into(
                prepared
                    .transaction_publish
                    .allocation_generation_read_set_in(image.writes, &image.persisted),
                counts,
            );
        }),
        resume,
    );
    let tally = match outcome {
        Layer0EnumerationOutcome::WholeStream(tally) => tally,
        Layer0EnumerationOutcome::OneShardWrittenToItsLedger(_written) => return None,
    };
    let read_set_counts =
        AllocationGenerationReadSetCounts::from_observer_counts(&tally.observer_counts);
    Some((tally, read_set_counts))
}

/// 全量那条流在进度文件名里的名字（断点续跑按「输入指纹 + 流名 + 计划哈希」分格）。
const FULL_ENUMERATION_STREAM_NAME: &str = "first_transaction_stream";
/// 每次写只取两态时的闭式：1 + Σ(2^|段| − 1)，A 那一段 26 个写（两个系统配置槽写 + 24 个单元写）占 2^26 − 1
/// （E142（第一个事务的干跑） 产物 `closed_form=` 那个数，登记表八整条流那一句）。
const FULL_STATES_WITH_TWO_STATES_PER_WRITE: u64 = 67_108_885;
/// 全量枚举的状态数（代码审阅第 4 条，用户 2026-09-27 定「原地覆写补第三态、全量也跑」）：这条流上取三态的只有 8 次系统配置槽写，
/// 四个系统配置槽段各 3² − 1、A 那一段 3² · 2²⁴ − 1，其余同两态，再加全部持久那一个。
const FULL_STATES: u64 = 150_994_980;
/// 甲二快档的状态数：A 那一段 2 原地写（系统配置槽写，各三态）× 单元写全不落或全落 3² · 2 − 1 = 17，四个系统配置槽段各 3² − 1 = 8，
/// 其余每段同全量（2 写段 3、1 写段 1），再加全部持久那一个（补第三态之前 29）。
const QUICK_TIER_STATES: u64 = 54;
/// 根槽 txg 3 已持久的状态：A 的根之后那一段（两块盘的系统配置槽轮换）3² − 1，再加全部持久那一个（补第三态之前 4）。
const ROOT_PERSISTED_STATES: u64 = 9;
/// 读出文件的状态：根已持久的那 [`ROOT_PERSISTED_STATES`] 个，加 A 的记录已落、根没落而恢复施加了记录的 3 个（补第三态之前 7）。
const FILE_READ_STATES: u64 = 12;
/// 环里一条记录都还没有的状态：取号那一段 3² − 1，加暖机第一次记录段整段没落那一个（补第三态之前 4）。
const STATES_WITH_AN_EMPTY_JOURNAL_RING: u64 = 9;

fn prepare(tag: &str) -> Prepared {
    let pool = build_pool(tag);
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 2, 1, 2, 2, 1, 26, 2, 1, 2]
    );
    assert_eq!(
        writes.len(),
        41,
        "取号 2 + 暖机两次各 5 + A 29（24 个单元写、两份记录、根槽、两块盘的系统配置槽轮换）"
    );
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

/// 全量：[`FULL_STATES`] 个状态。跑得慢（每个状态两遍恢复 + checker + 记录核对器），门禁 54 号在 release 下跑它；平时 `cargo test` 跳过。
/// 带断点续跑（`singlefs_harness::layer0_progress`）：进度目录、输入指纹与强制从头跑的开关都从环境变量取，没设进度目录就不留进度文件；
/// I-3.10 那一类的逐状态计数记在观察者计数里，续跑时随进度文件接上。双机分片（`SINGLEFS_LAYER0_SHARD`，
/// `research/scripts/layer0-shard-run.sh`）：跑一片时只写账本、不打计数行、不判；merge 那一趟拿并齐的计数照下面逐项判、逐字打同样的行。
#[test]
#[ignore = "全量六千七百多万个状态，门禁 54 号（.claude/gate.d/54-layer0-replay.sh）在 release 下跑，带断点续跑"]
fn layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations() {
    let prepared = prepare("layer0-full");
    assert_eq!(
        closed_form_state_count(&prepared.segments),
        FULL_STATES_WITH_TWO_STATES_PER_WRITE,
        "每次写只取两态时的闭式（E142 产物的 closed_form）"
    );
    // 计数行的 closed_form 报枚举域的闭式（原地覆写三态），54 号认 exhaustive=true 就是枚举到的恰好这么多。
    let closed_form = layer0_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let Some((tally, read_set_counts)) = enumerate_counting_allocation_generation_read_sets(
        &prepared,
        &full_expansion,
        &Layer0Resume::from_environment(FULL_ENUMERATION_STREAM_NAME),
    ) else {
        return;
    };
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
    // 读出文件、根槽已持久、journal 承重、验证跑过的那几格都在 A 那一段之后（记录段、根槽段、轮换段、全部持久），与甲二快档相同；
    // A 那一段的单元写只落一部分的状态上 A 的记录与根都没落，恢复落在暖机 txg 2 的根上、没有文件（实六推的，54 号全量核）。
    // 系统配置槽撕裂的状态（第三态）与那次写没持久的同一状态恢复得一样：撕裂的那一槽解不开，择的是同一块盘上另一槽，
    // 而没持久时那一槽里是更旧的一代、择的也是另一槽（推的，补第三态之后 54 号全量核）；它们不带待施加的记录，journal 承重、验证那几格不变。
    assert_eq!(
        oracle_counts(&tally),
        OracleCounts {
            states: FULL_STATES,
            violations: 0,
            root_persisted_states: ROOT_PERSISTED_STATES,
            no_file_states: FULL_STATES - FILE_READ_STATES,
            file_read_states: FILE_READ_STATES,
            failed_states: 0,
            journal_differing_states: 3,
            verification_ran_states: 6,
            verification_failed_states: 0,
            first_violation: None,
        }
    );
    assert_eq!(closed_form, FULL_STATES, "全量：枚举到的状态数等于闭式");
    // 按发布分（里程碑「第二个事务」步 6 验收第 1 条的口径，第一条流同样报）：暖机 txg 1、2 各 8 + 3 + 1（取号、轮换那一段各 3² − 1），
    // A 是 26 写段 3² · 2²⁴ − 1 = 150994943 + 记录段 3 + 根槽段 1，A 的系统配置槽轮换 8，再加全部持久那一个。
    assert_eq!(
        tally.states_by_publish_text(),
        "instance1_txg1=12 instance1_txg2=12 instance1_txg3=150994947 after_the_last_root=8 every_write_persisted=1"
    );
    assert_allocation_generation_read_set_reached_through_the_record(&read_set_counts);
    assert_checker_counts(
        &tally,
        FULL_STATES,
        ROOT_PERSISTED_STATES,
        STATES_WITH_AN_EMPTY_JOURNAL_RING,
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

/// 平时跑的那一份（甲二快档，用户 2026-09-26 定）：每段原地写各取它的几态、任意组合 × 单元写全不落或全落——A 那一段 26 个写只展开
/// 两个系统配置槽写（各三态）的组合各配单元写全不落或全落（17 个），其余九段全展开——[`QUICK_TIER_STATES`] 个状态；报出来的是「不是全量」。
#[test]
fn layer0_quick_tier_matches_the_full_tally_shape() {
    let prepared = prepare("layer0-partial");
    let (tally, read_set_counts) = enumerate_counting_allocation_generation_read_sets(
        &prepared,
        &quick_tier_expansion,
        &Layer0Resume::NoProgressFile,
    )
    .expect("不留进度文件就不分片，交回整条流的计数");
    println!("{}", checker_line(&tally));
    println!("CHECKER_FIRST {:?}", tally.checker_first_violation);
    assert_eq!(
        oracle_counts(&tally),
        OracleCounts {
            states: QUICK_TIER_STATES,
            violations: 0,
            root_persisted_states: ROOT_PERSISTED_STATES,
            no_file_states: QUICK_TIER_STATES - FILE_READ_STATES,
            file_read_states: FILE_READ_STATES,
            failed_states: 0,
            journal_differing_states: 3,
            verification_ran_states: 6,
            verification_failed_states: 0,
            first_violation: None,
        },
        "少展开的只有 A 那一段单元写只落一部分的组合：文件在 / 根已持久 / 验证跑过 / journal 承重的状态数都与全量相同"
    );
    assert_eq!(
        tally.states_by_publish_text(),
        "instance1_txg1=12 instance1_txg2=12 instance1_txg3=21 after_the_last_root=8 every_write_persisted=1",
        "甲二按发布分：少的只在 A 那一格（A 那一段 17 + 记录段 3 + 根槽段 1）"
    );
    assert_eq!(
        layer0_state_count_with_torn_in_place_overwrites(
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            &quick_tier_expansion
        ),
        QUICK_TIER_STATES,
        "甲二按闭式数"
    );
    assert_eq!(
        layer0_state_count(&prepared.writes, &prepared.segments, &quick_tier_expansion),
        29,
        "每次写只取两态时甲二的闭式（补第三态之前的数）"
    );
    assert_allocation_generation_read_set_reached_through_the_record(&read_set_counts);
    assert_checker_counts(
        &tally,
        QUICK_TIER_STATES,
        ROOT_PERSISTED_STATES,
        STATES_WITH_AN_EMPTY_JOURNAL_RING,
        read_set_counts.states_judging_allocation_generations(),
    );
    assert_eq!(
        closed_form_state_count(&prepared.segments),
        FULL_STATES_WITH_TWO_STATES_PER_WRITE
    );
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
/// oracle 对 12 个单元逐个都要判红；全部持久那一个判绿；根槽已持久而 journal 记录两份都没持久不算违例（根自己带着全部字段）。
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
    assert_eq!(unit_write_indices.len(), 24);
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
    assert_eq!(violations, 12, "12 个单元逐个抽掉，12 次都判红");
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
/// 摘掉第一个事务里「journal 记录 → 根槽」那道池屏障（录制器按设备记屏障，两块盘各一步，两步都摘），两份记录与根槽同段；
/// 枚举里出现根在案而记录一份都不在的状态。oracle 在这里不红。只摘盘 1 那一步的判别力在
/// `crash_segments_per_device_and_torn_in_place_overwrites.rs`（按 A 那一段并到系统配置槽轮换，5 个写）。
#[test]
fn removing_the_barrier_before_the_root_slot_is_caught_by_the_record_checker() {
    let pool = build_pool("record-checker-barrier-two");
    let base = pool.memory_pool_after_mkfs();
    let mut operations = pool.retained_operations()[pool.mkfs_operation_count..].to_vec();
    let root_position = operations
        .iter()
        .rposition(|retained| geometry().classify(&retained.operation) == StepKind::RootRecordFua)
        .expect("有根槽写");
    let first_barrier_before_the_root = operations[..root_position]
        .iter()
        .rposition(|retained| retained.operation.kind != RecordedOperationKind::Barrier)
        .expect("根槽之前有写")
        + 1;
    assert_eq!(
        root_position - first_barrier_before_the_root,
        2,
        "根槽之前那道池屏障两块盘各记一步"
    );
    operations.drain(first_barrier_before_the_root..root_position);
    let (writes, segments) = writes_and_segments(&operations, &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 2, 1, 2, 2, 1, 26, 3, 2],
        "记录两份与根槽并成一段"
    );
    let root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽");
    // A 那一段 26 个写不展开（只以整段持久进入后面的状态），其余每段任意子集。
    let tally = enumerate_layer0_selecting(
        &base,
        &writes,
        &segments,
        root_index,
        &file_content(),
        &|_segment_index, segment| segment.len() <= 3,
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
