//! checker 档模块：crash、crash_state_enumeration、crash_point_replay_progress、crash_given_states、crash_amplification
//! 里程碑「新池新建文件」步 7 的验收：拿步 5 的录制流按 D13（验证路线） 已定项 4 枚举全部崩溃状态，每个状态跑三件事——
//! 步 6 的恢复与 oracle、池级 checker（一元判决）、记录核对器（二元判决）。oracle 那一半的计数与 E142（新池新建文件的干跑）
//! 第八次跑产物的 `name=layer0` / `name=journal_effect` 行逐字对；再加一组靶向的阳性对照。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use common::{build_pool, file_content, geometry};
use singlefs_checker::image::{
    COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
    DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT,
    DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
    DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE,
    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY, EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
    EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY,
    FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
    INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
    LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
    LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT,
    MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
    NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
    NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
    ONE_OBJECT_BIRTH_PER_INODE_NUMBER, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
    RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
    TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
    UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS,
    UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
};
use singlefs_checker_tier::crash::{
    check_records, crash_point_replay_plan_state_count, crash_state_count,
    crash_state_count_with_torn_in_place_overwrites, evaluate_state, full_expansion,
    oracle_violation, quick_tier_expansion, CrashPointReplayObserverCounts,
    CrashPointReplaySegmentExpansion, CrashPointReplayTally, CrashPointReplayVersionTable,
};
use singlefs_checker_tier::crash_amplification::{
    one_test_changes_the_environment_at_a_time, GivenStatesEngine, GivenStatesItem,
};
use singlefs_checker_tier::crash_given_states::{
    GivenStateConclusion, GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
};
use singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_checker_tier::crash_state_enumeration::{
    enumerate_crash_states_selecting, enumerate_crash_states_with_expansion,
    sample_crash_states_on_the_cpu_against_the_version_table,
    sample_crash_states_on_the_cpu_with_threads_against_the_version_table, CrashPointReplayItem,
    CrashPointReplayObserver,
};

/// 这份测试文件的仓内路径：崩溃枚举用例的设点文件（节点身份与 CPU 结论版本按它取）。
const THIS_FILE: &str =
    "crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs";
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::recovery::RecoveryReport;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_harness::memory_pool::{
    closed_form_state_count, root_identity_written_by, writes_and_segments, CrashImage, MemoryPool,
    PublishedVersion, RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::RecordedOperationKind;

/// oracle 那一半：字段照 E142（新池新建文件的干跑） 产物的 `name=layer0` / `name=journal_effect` 两行（第八次跑那一份是 A 只有 16 个单元写时的形状：
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
    first_violation: Option<String>,
}

fn oracle_counts(tally: &CrashPointReplayTally) -> OracleCounts {
    OracleCounts {
        states: tally.states,
        violations: tally.violations,
        root_persisted_states: tally.root_persisted_states,
        no_file_states: tally.no_file_states,
        file_read_states: tally.file_read_states,
        failed_states: tally.failed_states,
        journal_differing_states: tally.journal_differing_states,
        first_violation: tally.first_violation.clone(),
    }
}

/// checker 与记录核对器那一半打成一行：每条不变量「评估过的状态数 / 判违例的状态数」。
fn checker_line(tally: &CrashPointReplayTally) -> String {
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
/// 那几条只在这些状态上评估（删除态扫描那几条 I-9.5、I-3.4、I-9.24、I-9.29、I-4.6、I-4.10、I-9.38 的理由写在函数里那张表旁边）
/// （I-9.25（树表条目的 previous_snapshot_txg）、I-9.27（树表条目的头 ID）、I-3.13（记账行的代段）、I-9.26（inode 记录的改动计数）、
/// I-5.5（从 mkfs 水位建树那一版的第一个数据单元） 与 I-3.5（引用区间的精确性） 是这个理由：种子根与暖机根指着的第 0 版树表是空的，
/// 只有 txg 3 那一版有树表条目、记账树、inode 记录与数据单元；记录施加出来的那一版由 I-4.1 另起一份走读去走、判定丢掉，不进这几条的计数，
/// 映射 key 与单元头相符（`MAPPING_KEY_MATCHES_THE_UNIT_HEADER`，实审 B2 新立、编号待定）判的是中央映射树叶里的每一条条目：
/// 种子根与暖机根的映射根指针全零（没有文件的一版），走不到条目；txg 3 的记录已落而根槽没落时，由记录施加出来的那一版
/// 在这条流上不走（它要实例表里有一行记着某次恢复施加到过 txg 3，实例表的行是挂载写行时写的，而这条流上唯一一次挂载在 txg 3 之前），
/// I-3.10 读下一次挂载会先施加的那一版只读树表、不走映射树，
/// I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉） 与 I-9.7 同一个理由——要走得到 inode 叶容器里的记录，
/// I-3.11（已分配减 defer 等于最新根走读） 与 I-3.1（已分配统计对得上） 同一个理由——要最新根下面的记账树：
/// 种子根与暖机根指着的第 0 版树表是空的，没有分配记录树；
/// I-1.10（码 2 条目宽等于字段表宽） 同一个理由：第 0 版树表一条条目都没有，没有哪个码 2 节点的条目宽可比）；
/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 单列一类：它除了回退候选集里那几版，还读下一次挂载会先施加的那一版
/// （`crates/singlefs-checker/invariants/05-空间记账.md` I-3.10 那一行射程 ④），txg 3 的记录至少一份已落、根槽没落的状态上也评估，
/// 评估过的状态数由调用方按录制流逐状态现算（[`TransactionPublishInRecordedStream::allocation_generation_read_set_in`]），不写死；
/// I-5.4（分配记录罩住的槽互不相交） 在这一类里：按版本那一层也判下一次挂载先施加的那一版（第 34 批改法 RV，`crates/singlefs-checker/invariants/07-分配器.md`
/// I-5.4 那一行），两条认这一版用的是池级 checker 里同一个函数，这条流上评估过的状态逐个相同，理由写在函数里那张表旁边；
/// I-9.14（树表条目的诞生 txg 跨根不变）在这条流上一个状态都评估不到，I-9.28（全部读得出的根之间同一树 ID 的诞生 txg 相同） 同一个理由；
/// I-3.12（实例表行 T 低于引用它的根的 txg） 也评估不到：这条流上没有哪次发布重写实例表（取号只写系统配置，暖机与 A 照抄
/// mkfs 那张只有链记录、没有行的实例表）；
/// I-8.6（反向链算法） 要环里至少有一条自证过的记录，环还空着的那几个状态上报「不适用」；
/// I-8.7（实例内事务号不重号）、I-8.8（前缀里的事务不被切开） 与 I-7.9（回退下界 F 不高于抬 F 的上限） 在这条流上一个状态都评估不到；
/// I-8.10、I-8.11、I-8.12、I-4.1、I-7.16 各自一类，评估过的状态数逐条写在函数里那几张表旁边；
/// 删除与回退那九条（I-4.7、I-4.4 与墓碑、谱系那七条 I-9.17–I-9.23）与退役那四条（I-9.31–I-9.34）在这条流上一个状态都评估不到，
/// I-4.6、I-4.10、I-9.38（截断那一批起每条记录都判）与 I-9.39（截断第二轮新立，每条记录判一次）与删除态扫描那几条一样只在根已落的状态上评估，
/// I-9.35（退役完备）只在根已落的状态上评估，理由同样写在表旁边；I-9.40（extent 上段形状由有条目的 inode 号集合定）判在删除态扫描里、
/// 与 I-9.5 同一份版本清单，同样只在根已落的状态上评估；
/// 其余 34 条每个状态都评估（第 50 批合后 checker 档快档那一行 `CHECKER`：90 条里 34 条是 46/0）；
/// 里程碑四进清单的那几条：I-3.6（deadlist 紧界）、I-3.7（活快照可枚举） 每个状态都评估，I-3.15（未释放记录罩住的单元由映射指在这一槽）
/// 与 I-3.16（碎片度与全空段统计对得上） 只在根已落的状态上评估，I-5.6（活整理意图的区域里没有签发之后分配的记录） 与 I-1.14（同一映射 key 在各版里的单元逐字节相同）
/// 一个状态都评估不到，I-1.13（扫描重建与遍历方向的索引相同） 照 CPU 判法每个状态都评估、GPU 判器那一趟不比，理由逐条写在函数里那几张表旁边。
/// 违例只许出现在 I-7.7（系统配置实例代号不低于根环）：C322（取号那一步的屏障怎么放没有条款） 未还，
/// 取号只落了一块盘的那 2 个状态里两份系统配置的实例代号不等。
fn assert_checker_counts(
    tally: &CrashPointReplayTally,
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
        "I-9.25",
        "I-9.27",
        "I-3.13",
        "I-9.26",
        "I-5.5",
        "I-3.5",
        // 删除态扫描（`walk::judge_deletion_state_of_the_versions` 与 `judge_tombstones_and_the_lineage_of_the_versions`）只扫树表有条目的那几版，
        // 这条流上只有 txg 3 那一版：I-9.5（队列与 nlink 0 的记录同号）与 I-9.24（队列至多一条 key，空队列报成立）逐条判 `deleted_inodes` 条目，
        // I-3.4（记账标签 4 恒 0）判记账树，I-9.29（同号只许一个出生代）比 inode 记录收出来的号——都要 txg 3 的根已落。
        // 记录已落、根没落的那几个状态上由记录施加出来的那一版不进删除态扫描的版本清单（同 I-1.11 那一行的理由：实例表里没有行）。
        // I-9.5、I-3.4 的 9 对得上 2026-10-05 主工作区 checker 档日志快档那一行的 `I-9.5=9/0`、`I-3.4=9/0`；I-9.24、I-9.29 是推的。
        "I-9.5",
        "I-3.4",
        DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
        ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
        // I-9.40（extent 上段形状由有条目的 inode 号集合定，`walk::judge_extent_upper_segment_shape_of_a_version`）判在删除态扫描里，
        // 与 I-9.5 同一份版本清单、同一道「树表 0 条不判」：种子根与暖机根的第 0 版树表 0 条、不判也不报部分没判；
        // txg 3 那一版树表里有一棵根指针不全零的 extent 树（t2），根已落的状态上 A 的单元全落了、上段读得全，每个这样的状态判一次。
        // 记录已落、根没落的那几个状态上由记录施加出来的那一版不进版本清单（同 I-9.5）。快档 9 对得上主工作区崩溃轻型这条流快档的
        // `I-9.40` 评估数 9（那一趟用例把它算进「其余」、期望 46 而红；取号写一轮时量的，取号写两轮不改根已落的状态数）。
        EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY,
        // I-4.6（单元连号、单元数对标签）、I-4.10（截断中间态有账） 与 I-9.38（数据单元声明长度对 size） 自截断起在删除态扫描里每条记录都判
        // （判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K7 T-2 到 T-4），与 I-9.5 同一个读集：txg 3 那一版的 inode 树要根已落；
        // 根已落的状态上 A 的单元全落了（A 的单元写排在记录段与根槽段之前），I-9.38 读得出 size 之内的每个单元。推的，没在 checker 档跑过。
        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
        DATA_UNITS_BEYOND_THE_SIZE_ARE_NAMED_BY_A_TRUNCATION_INTENT,
        DECLARED_LENGTH_OF_EACH_DATA_UNIT_WITHIN_THE_SIZE_MATCHES_THE_SIZE,
        // I-9.39（截断字段之外的活单元不早于它，判决 `research/prompts/m4-trunc-r2-main-verification.md` 第三节 R6）同一个读集、每条记录判一次：
        // 这条流上记录的截断字段都是 0，判过而成立。推的，没在 checker 档跑过。
        LIVE_DATA_UNITS_BEYOND_THE_TRUNCATION_FIELD_ARE_BORN_NO_EARLIER_THAN_IT,
        // I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上，第 49 批）只在最新根下面有走全了的记账树时判：txg 3 那一版自己写的那一行
        // 就是第一级台阶 (3, 2)，① 判得到、成立；它的谱系树、墓碑树都是零根。种子根与暖机根没有记账树，报不适用。推的，没在 checker 档跑过。
        "I-9.30",
        // I-9.35（退役完备，第 50 批）比相邻两版：删除态扫描只扫到 txg 3 那一版有 inode 树（要它的根已落），它的父版取 txg − 1 = 2
        // （谱系树零根，没有 T_rb = 3 的记录），暖机第二次那条根比它早一段落盘、根已落的状态上恒在回退候选集里，树表 0 条、当父版时叶身份集是空集；
        // txg 3 那一版的 inode 树一片叶、退役树零根：父版少的叶 = 空集 = 死于 txg 3 的退役记录 ⇒ 判得到、成立。
        // 根没落的状态上没有一版有 inode 树，报不适用。对得上第 50 批合后 checker 档快档那一行 `CHECKER` 的 `I-9.35=9/0`。
        RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
        // I-3.15（未释放记录罩住的单元由映射指在这一槽，`walk::judge_unreleased_allocation_records_against_the_central_mapping`）逐版判
        // 回退候选集里各版分配记录树里的未释放记录：这条流上只有 txg 3 那一版有分配记录树，它要在候选集里（根已落）；
        // 由记录施加出来的那一版同 I-1.11 那一行的理由不进版本清单。根没落的状态上一条记录都走不到，报不适用。
        UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
        // I-3.16（碎片度与全空段统计对得上，`walk::judge_fragmentation_and_empty_segment_statistics`）与 I-3.1 同在「最新根下面有记账树」那一支里：
        // 根没落的状态上最新根是暖机或种子根、第 0 版树表没有记账树，报不适用；根已落的状态上 I-3.1 判成立（占着的槽数乘 16 KiB 等于记账的已分配）、
        // txg 3 那一版读得到分配记录树，runs 与按记录口径的全空段数两半都判。这是 CPU 现在的判法（读到的分配记录树 0 棵才报部分没判，
        // 没有「两个口径不等就不判」那道门）；GPU 那道门去掉之前它在这条流上判出的也是 9（这些状态上两个口径相等）。
        FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
    ];
    // I-3.10 除了回退候选集，还读下一次挂载会先施加的那一版（射程 ④）：新根没落而 txg 3 的记录已落时，
    // 恢复要由那条记录重建 txg 3 那一版，它的分配记录树在记录的新根段指着的树表下面。
    // I-5.4 按版本那一层同样判这一版（第 34 批改法 RV）。两条在这条流上逐状态评估同一批状态（推的，读码，没在 checker 档跑过）：
    // ① 认这一版两条都经 `walk::version_the_next_mount_applies_first`（I-3.10 经 `tree_table_pointer_of_the_version_the_next_mount_applies_first`
    // 取它的树表指针），候选根两条都取 `candidate_indexes`，由记录施加出来的那几版两条都取、这条流上一版都没有；
    // ② 这条流上只有 txg 3 那一版有分配记录树，A 的 24 个单元写自成一段、排在记录段与根槽段之前（`prepare` 里钉的段长），
    // txg 3 的记录或根落了的状态上它的单元全落了：树表、分配记录树读得出、走得到叶（I-5.4 判成立），记录罩住的单元头都可用（I-3.10 判得到记录）；
    // ③ 种子根与暖机根指着的第 0 版树表在 mkfs 的基线里、读得出、0 条，根记录里树表 0 条那一版的分配记录树根全零：
    // I-5.4 那几格「树表读不出」「树表指针全零」「根记录直接持有的树读不出」都不出现，不报部分没判；txg 3 那一版不在读集里的状态上两条都报不适用。
    let under_the_new_root_or_through_the_record_the_next_mount_applies_first = ["I-3.10", "I-5.4"];
    // I-9.14 要同一棵树的条目出现在两个树表单元里才比得出来，而这条流上只有新池新建文件写树表：mkfs 种的第 0 版树表是空的、
    // 两次暖机空发布不写树表 ⇒ 每个状态都报「不适用」，一个状态都评估不到。跨根比得出来的流在
    // `crash_enumeration_fixed_script_stream.rs`（发布 B 起每次发布都重写树表），那里它真被评估过。
    // I-8.7 要同一个实例写出两条非 0 事务号的记录才比得出来，而这条流上实例 1 只发了一个承载事务的版本
    // （事务号 1）：写行与两次暖机都是事务号 0 的空发布（D23（journal 的角色与格式） 已定项 19 ①），不进那个序列
    // ⇒ 每个状态都报「不适用」，一个状态都评估不到。比得出来的流在 `crash_enumeration_fixed_script_stream.rs`
    // （发布 B 起同一实例接着往上数事务号），那里它进 `must_evaluate`、真被评估过。
    // I-8.8（前缀里的事务不被切开） 要同一个事务号落了两条以上的记录（③）、或者有一组一条提交标记都没有（④）才有对象，
    // 而第一版一事务一条、每条都带提交标记 ⇒ 每个状态都报「不适用」（提交标记字节照样判、判出别的值照样红，
    // 违例数仍由下面那条断言钉 0）。它在哪一条流上都评估不到，判别力在 `checker_known_bad_images.rs` 的阳性对照与坏镜像上。
    // I-7.9（回退下界 F 不高于抬 F 的上限） 只判抬 F 的那一条根，这条流上没抬过 F（每条根带的 F 都是 0）⇒ 每个状态都报「不适用」。
    // 抬过 F 的流在 `crash_enumeration_fixed_script_stream.rs`（E 之前抬到 11），那里它真被评估过。
    // I-9.28 要两条根的树表里有同一个树 ID 的条目才比得出来，这条流上只有 txg 3 那条根的树表有条目 ⇒ 每个状态都没有对象。
    // I-3.12 要根指着的实例表里有行，这条流上全部根都指着 mkfs 那张只有链记录的实例表（取号不写行、暖机与 A 照抄实例表指针）。
    // 两条的 0 都对得上 2026-10-04 主工作区 checker 档日志快档那一行 `CHECKER` 的 `I-9.28=0/0`、`I-3.12=0/0`。
    let never_comparable_on_this_stream = ["I-9.14", "I-8.7", "I-8.8", "I-7.9", "I-3.12", "I-9.28"];
    // 这条流上没有删除（没有 nlink = 0 的记录、`deleted_inodes` 里没有号）：I-4.7（删除中 inode 有账） 没有对象；
    // I-9.18（删除队列里的号有谱系上的墓碑） 队列空就不判。I-4.7 的 0 对得上同一份日志的 `I-4.7=0/0`（I-9.18 是推的）。
    // I-4.6 自截断起每条记录都判，挪到 `only_under_the_new_root`。
    let never_has_an_inode_being_deleted_on_this_stream =
        ["I-4.7", QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE];
    // 没有删除、没有回退：txg 3 那一版的墓碑树、谱系树都是零根（全部持久那个状态就是 `checker_known_bad_images.rs` 的干净镜像，
    // 那里这几条报不适用），主走读走不到它们的内部条目（I-9.17、I-9.23），删除态扫描读出的墓碑容器是空的（I-9.19、I-9.20 没有墓碑可判，
    // I-9.21 在旧入口上要至少两片容器）；I-9.22 只在新入口装了旋钮时判，崩溃点重放走旧入口 `check_pool_image`。全是推的。
    let never_has_a_tombstone_or_lineage_record_on_this_stream = [
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
        NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
    ];
    // 没有删除：没有 inode 叶容器删空、被吸收，txg 3 那一版的退役树是零根（第 50 批，开池就建、零根注册）。主走读走不到退役树的内部条目
    // （I-9.31 照 I-9.17、I-9.23 的形态：一条都没走到报不适用），删除态扫描读出的退役容器是空的（I-9.32 没有记录可判，
    // I-9.33、I-9.34 没有死亡 txg 在谱系上的记录）。四条的 0 对得上第 50 批合后 checker 档快档那一行 `CHECKER` 的
    // `I-9.31=0/0`、`I-9.32=0/0`、`I-9.33=0/0`、`I-9.34=0/0`。
    let never_has_a_retirement_record_on_this_stream = [
        RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
        EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
        RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
        RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
    ];
    // 谱系树与退役树的容器容量那两条（第 53 批）：这条流上两棵都是零根，没有非最右的容器（I-9.36 没有对象）；崩溃点重放走池级 checker 的旧入口，
    // 没有谱系与退役的旋钮值（I-9.37 一格都不判）。推的：两条都是 0/0，没跑。
    let never_has_two_lineage_or_retirement_containers_on_this_stream = [
        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
        LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
    ];
    // I-4.4（意图只描述目标态） 第一版报不适用（用户 2026-10-05 定），哪一条流上都评估不到；对得上同一份日志的 `I-4.4=0/0`。
    let not_applicable_in_the_first_version = [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE];
    // 这条流上没有整理、也只有一版有中央映射：I-5.6（活整理意图的区域里没有签发之后分配的记录） 要某一版 logged_ops 树里有 R 对齐的整理意图，
    // 这条流上一版 logged_ops 都没写过意图；I-1.14（同一映射 key 在各版里的单元逐字节相同） 要同一把 key 在两版里落在不同的位置条目上，
    // 种子根与暖机根的映射根指针全零，只有 txg 3 那一版有映射条目。两条每个状态都报不适用。
    let never_has_a_compaction_intent_or_two_versions_of_one_mapping_key_on_this_stream = [
        COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
        UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS,
    ];
    // I-3.6（deadlist 紧界）、I-3.7（活快照可枚举） 落在「其余」那一类、每个状态都评估：一条 deadlist 条目、一条码 14 条目都没有的一版报成立
    // （同 I-9.24 空队列报成立）。I-3.6、I-3.7、I-3.15、I-3.16、I-5.6、I-1.14 这几个数对得上 2026-10-15 崩溃轻型那一趟这条流快档的
    // `CHECKER` 行（GPU 判器判：`I-3.6=46/0 I-3.7=46/0 I-3.15=9/0 I-5.6=0/0 I-3.16=9/0 I-1.14=0/0`）；这几位 GPU 与 CPU 对拍逐位比，
    // CPU 那一趟的数是照 `walk` 那几个函数推的、没在 CPU 判器上跑过这条流。
    // I-1.13（扫描重建与遍历方向的索引相同） 照 CPU 判法落在「其余」那一类：最新根读得出、实例表读得全，遍历方向那三张表与扫描重建的三张表
    // 每个状态都比得了（推的，没在 CPU 判器上跑过这条流）。GPU 判器的扫描重建那一截补上之后（C821）两路照同一个判法判、计数一样，不再认 0。
    // I-8.6 判的是「本实例内逻辑前一条」与这一条的链值，环里一条自证过的记录都没有时判不了：
    // 取号两轮各一段（每段两盘各一次系统配置写，原地覆写各三态）各 3² − 1 个状态，加上暖机第一次的记录段整段没落那一个，
    // 一共 17 个状态还没有记录落盘（取号写一轮时 9 个，补第三态之前 4 个）。
    // w1 的记录一落盘（两盘任一份）就判得了：它的计数器是 1 ⇒ 本实例第一条 ⇒ 反向链恒 0。
    // I-8.9（一次发布的记录序号连续且只有末条带标志） 同一个理由：环里有一条自证过的记录就判得了（记录标志其余位、序号 0 逐条判），
    // 没有就报不适用；这条流上 w1、w4、t9 各是一次只有一条记录的发布（序号 1、带末条标志）。
    let judged_once_a_record_is_on_disk = ["I-8.6", "I-8.9"];
    // I-8.10 拿带末条标志的记录的新根段去比根环里同 (实例, txg) 的根：环里有记录还不够，还要它那次发布的根已落。
    // 环空着的 [`STATES_WITH_AN_EMPTY_JOURNAL_RING`] 个之外，暖机第一次的记录落了而它的根没落的
    // [`STATES_WITH_ONLY_RECORDS_OF_THE_FIRST_WARM_UP_BEFORE_ITS_ROOT`] 个也没有可比的根（mkfs 的种子根是实例 0、txg 0）。
    let judged_once_a_record_and_the_root_of_its_publish_are_on_disk = ["I-8.10"];
    // I-8.11 判点名项与被指单元的头：只有 A 的记录点名单元（暖机是零单元的空发布），A 的记录一份落了才有对象。
    let judged_once_a_record_of_the_file_publish_is_on_disk = ["I-8.11"];
    // I-8.12 比同一块盘上同一实例前后两条记录的 txg：一块盘上要有两条实例 1 的记录（暖机第二次的记录落了）才有对象。
    let judged_once_a_device_holds_two_records_of_one_instance = ["I-8.12"];
    // I-4.1 走重放之后那一版：所选根之上一条都不施加时报不适用。
    let judged_only_when_the_replay_applies_a_record = ["I-4.1"];
    // I-7.16 判可择的系统配置槽的世代号：每块盘都只剩一槽可择、而且是 mkfs 写下的世代号 1 时一格都不判。
    let judged_unless_every_device_has_one_choosable_slot_written_by_mkfs = ["I-7.16"];
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        let expected_evaluated = if never_comparable_on_this_stream.contains(&invariant)
            || never_has_an_inode_being_deleted_on_this_stream.contains(&invariant)
            || never_has_a_tombstone_or_lineage_record_on_this_stream.contains(&invariant)
            || never_has_a_retirement_record_on_this_stream.contains(&invariant)
            || never_has_two_lineage_or_retirement_containers_on_this_stream.contains(&invariant)
            || not_applicable_in_the_first_version.contains(&invariant)
            || never_has_a_compaction_intent_or_two_versions_of_one_mapping_key_on_this_stream
                .contains(&invariant)
        {
            0
        } else if only_under_the_new_root.contains(&invariant) {
            root_persisted_states
        } else if under_the_new_root_or_through_the_record_the_next_mount_applies_first
            .contains(&invariant)
        {
            states_judging_allocation_generations
        } else if judged_once_a_record_is_on_disk.contains(&invariant) {
            every_state - states_with_an_empty_journal_ring
        } else if judged_once_a_record_and_the_root_of_its_publish_are_on_disk.contains(&invariant)
        {
            every_state
                - states_with_an_empty_journal_ring
                - STATES_WITH_ONLY_RECORDS_OF_THE_FIRST_WARM_UP_BEFORE_ITS_ROOT
        } else if judged_once_a_record_of_the_file_publish_is_on_disk.contains(&invariant) {
            root_persisted_states + STATES_WITH_A_RECORD_OF_THE_FILE_PUBLISH_BUT_NOT_ITS_ROOT
        } else if judged_once_a_device_holds_two_records_of_one_instance.contains(&invariant) {
            every_state - STATES_BEFORE_A_DEVICE_HOLDS_TWO_RECORDS_OF_ONE_INSTANCE
        } else if judged_only_when_the_replay_applies_a_record.contains(&invariant) {
            STATES_WHERE_THE_REPLAY_APPLIES_A_RECORD
        } else if judged_unless_every_device_has_one_choosable_slot_written_by_mkfs
            .contains(&invariant)
        {
            every_state - STATES_WHERE_EVERY_DEVICE_HAS_ONE_CHOOSABLE_SLOT_WRITTEN_BY_MKFS
        } else {
            every_state
        };
        let evaluated = tally
            .checker_evaluated_states
            .get(invariant)
            .copied()
            .unwrap_or(0);
        assert_eq!(
            evaluated, expected_evaluated,
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

/// 这一状态上，承载新池新建文件的那一版（txg 3）经哪条路进 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 的读集。
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

/// 录制流里承载新池新建文件的那次发布：它的根槽写之前、上一条根槽写之后有单元写（两次暖机是零单元的空发布）。
/// I-3.10 那一类评估过的状态数按它逐状态现算：只看录制流里的写种类、写次序、根槽写里写下的根身份与这一状态里哪几次写持久了，
/// 不读盘、不经 checker 的代码（`crates/singlefs-checker/invariants/README.md`「状态」一节「与实现只共享常量模块」：判的一方不许与被判的一方共用同一段代码）。
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
                StepKind::ZeroFill
                | StepKind::Discard
                | StepKind::SystemConfigurationSlot
                | StepKind::Barrier => {}
            }
        }
        assert_eq!(
            transaction_publishes.len(),
            1,
            "这条流上写单元的发布只有新池新建文件那一次"
        );
        let (root_write_index, record_write_indexes) = transaction_publishes.remove(0);
        assert!(
            !record_write_indexes.is_empty(),
            "新池新建文件那次发布写了 journal 记录"
        );
        let (checkpoint_txg, instance) = root_identity_of(&writes[root_write_index]);
        let previous_root_identity = (
            CheckpointTxg(
                checkpoint_txg
                    .0
                    .checked_sub(1)
                    .expect("新池新建文件之前有两次暖机发布，它的 txg 不是 0"),
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

    /// 观察者另读的这几样（根槽写与记录写的下标、前一版的根身份）的摘要：进观察者的版本，录制流一变它就变，判过的块不按旧的复用。
    fn extra_inputs_digest(&self) -> [u8; singlefs_harness::sha256::DIGEST_BYTES] {
        singlefs_harness::sha256::sha256_digest(
            format!(
                "root_write_index={}\nrecord_write_indexes={:?}\nprevious_root_identity={:?}\n",
                self.root_write_index, self.record_write_indexes, self.previous_root_identity
            )
            .as_bytes(),
        )
    }

    /// 崩溃点重放的写要么整条落、要么整条不落：落了而没被更晚的已持久写盖过的那份记录，盘上字节就是录制流里写下的那一条，照原样自证。
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

/// 观察者计数里这两类的名字（按块存进库、判段重建，`crash::CrashPointReplayObserverCounts`）。
const STATES_THROUGH_THE_LANDED_ROOT: &str = "states_through_the_landed_root";
const STATES_THROUGH_THE_RECORD_THE_NEXT_MOUNT_APPLIES_FIRST: &str =
    "states_through_the_record_the_next_mount_applies_first";

impl AllocationGenerationReadSetCounts {
    /// 一个状态记进观察者计数：从库里复用的块不再给观察者看，这两类的累计按块存在库里。
    fn count_into(
        read_set: TransactionVersionInAllocationGenerationReadSet,
        counts: &mut CrashPointReplayObserverCounts,
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

    fn from_observer_counts(counts: &CrashPointReplayObserverCounts) -> Self {
        Self {
            states_through_the_landed_root: counts.count_named(STATES_THROUGH_THE_LANDED_ROOT),
            states_through_the_record_the_next_mount_applies_first: counts
                .count_named(STATES_THROUGH_THE_RECORD_THE_NEXT_MOUNT_APPLIES_FIRST),
        }
    }

    fn states_judging_allocation_generations(&self) -> u64 {
        self.states_through_the_landed_root
            + self.states_through_the_record_the_next_mount_applies_first
    }
}

/// 单版本形态的 oracle 那一份版本表：被判的那次根槽写出的那一代就是唯一带文件的版本（与 `enumerate_crash_states_selecting` 里造的相同）。
fn single_version(prepared: &Prepared) -> Vec<PublishedVersion> {
    let (checkpoint_txg, instance) = root_identity_of(&prepared.writes[prepared.root_index]);
    vec![PublishedVersion {
        instance,
        checkpoint_txg,
        content: file_content(),
    }]
}

/// 观察者「分配代读集」（`allocation_generation_read_sets`）：这个状态上逐个现算 I-3.10 那一类，记进观察者计数。观察者的身份是这个 fn 的
/// 闭包键（入口登记在 `singlefs_checker_tier::judging_units_entries::OBSERVER_ENTRIES`）。
fn count_the_allocation_generation_read_set(
    prepared: &Prepared,
    image: &CrashImage<'_>,
    counts: &mut CrashPointReplayObserverCounts,
) {
    AllocationGenerationReadSetCounts::count_into(
        prepared
            .transaction_publish
            .allocation_generation_read_set_in(image.writes, &image.persisted),
        counts,
    );
}

/// 按段枚举（`expansion` 决定每一段怎么展开），每个状态上逐个现算 I-3.10 那一类，记进观察者计数（按块存进库，判段重建）。
/// 只录那一趟与只判自己那一份的那一趟交回 `None`：整条流的计数等判完全部块的那一趟。
fn enumerate_counting_allocation_generation_read_sets(
    prepared: &Prepared,
    item_label: &str,
    expansion: &dyn Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion,
) -> Option<(CrashPointReplayTally, AllocationGenerationReadSetCounts)> {
    let versions = single_version(prepared);
    let observe = |_ordinal: u64,
                   image: &CrashImage<'_>,
                   _consulted_report: &RecoveryReport,
                   counts: &mut CrashPointReplayObserverCounts| {
        count_the_allocation_generation_read_set(prepared, image, counts);
    };
    let item_name = format!("new_pool_file_creation::{item_label}");
    let tally = enumerate_crash_states_with_expansion(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.root_index,
        CrashPointReplayVersionTable::with_a_file_only(&versions),
        expansion,
        &[CrashPointReplayObserver {
            name: "allocation_generation_read_sets",
            extra_inputs_digest: extra_inputs_digest_that_was_computed(
                prepared.transaction_publish.extra_inputs_digest(),
            ),
            observe: &observe,
        }],
    )
    .judged()?;
    let read_set_counts =
        AllocationGenerationReadSetCounts::from_observer_counts(&tally.observer_counts);
    Some((tally, read_set_counts))
}

/// 崩溃点重放取样多线程与单线程的对拍（`.claude/rules/test-script-discipline.md`「编译与算法都要快」：多线程与单线程的对拍进快档）：
/// 同一条流、同一个序号段（整条流的前 20 个状态，三条线程切成不止一段），单线程那一支与三条线程那一支交回的都是整条流的状态数，
/// 每个序号各判一次，逐序号的红项与两遍恢复的结局逐字相同。判别力：多线程那一支交回判了几个、或把各段看到的整条流状态数相加
/// （`crates/mutations.tsv` 那两行）。
#[test]
fn sampling_with_three_threads_returns_the_whole_flow_state_count_and_judges_each_ordinal_once_like_one_thread(
) {
    let prepared = prepare("crash-point-replay-threaded-sampling");
    let versions = single_version(&prepared);
    let whole_flow_states = crash_point_replay_plan_state_count(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &quick_tier_expansion,
    );
    let sampled = 0..whole_flow_states.min(20);
    assert!(
        sampled.end >= 3,
        "序号段要切得出不止一段：{sampled:?}（整条流 {whole_flow_states} 个状态）"
    );
    let item = CrashPointReplayItem {
        name: "new_pool_file_creation::threaded_sampling",
        setting_file: THIS_FILE,
    };
    let judgement_words = |judgement: &singlefs_checker_tier::crash::CrashStateJudgement| {
        format!(
            "consulted={:?} ignored={:?}",
            judgement.consulted.outcome, judgement.ignored.outcome
        )
    };
    let mut by_one_thread: Vec<(u64, Vec<String>, String)> = Vec::new();
    let Some(one_thread_states) = sample_crash_states_on_the_cpu_against_the_version_table(
        item,
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.root_index,
        CrashPointReplayVersionTable::with_a_file_only(&versions),
        &quick_tier_expansion,
        sampled.clone(),
        &mut |ordinal, _image, judgement| {
            by_one_thread.push((ordinal, judgement.red_items(), judgement_words(judgement)))
        },
    )
    .judged() else {
        // 只录那一趟：流写进了判定库，不判
        return;
    };
    let by_three_threads: std::sync::Mutex<Vec<(u64, Vec<String>, String)>> =
        std::sync::Mutex::new(Vec::new());
    let Some(three_thread_states) =
        sample_crash_states_on_the_cpu_with_threads_against_the_version_table(
            item,
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            prepared.root_index,
            CrashPointReplayVersionTable::with_a_file_only(&versions),
            &quick_tier_expansion,
            sampled.clone(),
            std::num::NonZeroUsize::new(3).expect("3 不是 0"),
            &|ordinal, _image, judgement| {
                by_three_threads.lock().expect("判定线程不 panic").push((
                    ordinal,
                    judgement.red_items(),
                    judgement_words(judgement),
                ))
            },
        )
        .judged()
    else {
        return;
    };
    assert_eq!(
        one_thread_states, whole_flow_states,
        "单线程那一支交回整条流的状态数"
    );
    assert_eq!(
        three_thread_states, whole_flow_states,
        "三条线程那一支交回的也是整条流的状态数：不是各段的和，也不是判了几个"
    );
    assert_eq!(
        by_one_thread
            .iter()
            .map(|(ordinal, _, _)| *ordinal)
            .collect::<Vec<u64>>(),
        sampled.clone().collect::<Vec<u64>>(),
        "单线程那一支按序号各判一次"
    );
    let mut by_three_threads = by_three_threads.into_inner().expect("判定线程不 panic");
    by_three_threads.sort_by_key(|(ordinal, _, _)| *ordinal);
    assert_eq!(
        by_three_threads, by_one_thread,
        "三条线程那一支逐序号的红项与两遍恢复的结局与单线程逐字相同"
    );
}

/// 每次写只取两态时的闭式：1 + Σ(2^|段| − 1)，A 的 24 个单元写自成一段占 2^24 − 1。取号写两轮（D22（单元原子性怎么合成） 已定项 8 落地，
/// 每轮两盘各一写、轮后池屏障）各成一段，C577（发布返回之前、系统配置轮换之后加一道屏障）之后暖机第二次的轮换那 2 写自成一段：
/// 1 + (3 + 3 + 3 + 1 + 3 + 3 + 1 + 3 + 3 + 1 + 3) + (2^24 − 1) = 16777243（取号写一轮时 16777240；
/// E142（新池新建文件的干跑） 产物 `closed_form=` 那个数要跟着重出）。
const FULL_STATES_WITH_TWO_STATES_PER_WRITE: u64 = 16_777_243;
/// 全量枚举的状态数（代码审阅第 4 条，用户 2026-09-27 定「原地覆写补第三态、全量也跑」）：这条流上取三态的只有 10 次系统配置槽写，
/// 五个系统配置槽段（取号两轮各一段、三次轮换各一段）各 3² − 1、A 的单元段 2²⁴ − 1，其余同两态，
/// 再加全部持久那一个：1 + 5 × 8 + 3 × 3 + 3 × 1 + (2²⁴ − 1) = 16777268（取号写一轮时四个系统配置槽段，16777260）。
const FULL_STATES: u64 = 16_777_268;
/// 甲二快档的状态数：五个系统配置槽段各 3² − 1 = 8，A 的单元段只有单元写、全不落那一个 1，其余每段同全量（2 写段 3、1 写段 1），
/// 再加全部持久那一个：1 + 5 × 8 + 1 + 3 × 3 + 3 × 1 = 54（补第三态之前 1 + 5 × 3 + 1 + 3 × 3 + 3 × 1 = 29；取号写一轮时 46、两态 26）。
const QUICK_TIER_STATES: u64 = 54;
/// 根槽 txg 3 已持久的状态：A 的根之后那一段（两块盘的系统配置槽轮换）3² − 1，再加全部持久那一个（补第三态之前 4）。
const ROOT_PERSISTED_STATES: u64 = 9;
/// 读出文件的状态：根已持久的那 [`ROOT_PERSISTED_STATES`] 个，加 A 的记录已落、根没落而恢复施加了记录的 3 个（补第三态之前 7）。
const FILE_READ_STATES: u64 = 12;
/// 环里一条记录都还没有的状态：取号两轮各一段、各 3² − 1，加暖机第一次记录段整段没落那一个（取号写一轮时 9，补第三态之前 4）。
const STATES_WITH_AN_EMPTY_JOURNAL_RING: u64 = 17;
/// 环里只有暖机第一次（实例 1、txg 1）的记录、它的根还没落的状态：那一段两份记录只落一份的 2 个，加它的根槽段（根没落）那 1 个。
/// 快档与全量相同（A 的单元段在它们之后）。快档 I-8.10 = 54 − 17 − 3 = 34：取号多出来的 8 个状态环都空着，评估数不变，
/// 仍对得上 2026-10-04 主工作区 checker 档日志 `I-8.10=34/0`（那一趟取号写一轮，46 − 9 − 3）。
const STATES_WITH_ONLY_RECORDS_OF_THE_FIRST_WARM_UP_BEFORE_ITS_ROOT: u64 = 3;
/// A 的记录至少一份落了、A 的根没落的状态：记录段两份只落一份的 2 个，加根槽段（根没落）那 1 个。快档与全量相同（A 的单元段只在
/// 记录之前展开，那里记录一份都没落）。I-8.11 = 9 + 3 = 12，对得上同一份日志的 `I-8.11=12/0`。
const STATES_WITH_A_RECORD_OF_THE_FILE_PUBLISH_BUT_NOT_ITS_ROOT: u64 = 3;
/// 哪块盘上都还没有两条实例 1 的记录的状态：取号两段各 8、暖机第一次的记录段 3、根槽段 1、轮换段 8，加暖机第二次记录段整段没落那 1 个
/// （取号写一轮时 21）。快档与全量相同（A 的单元段在它们之后）。快档 I-8.12 = 54 − 29 = 25，评估数不变，
/// 仍对得上同一份日志的 `I-8.12=25/0`（那一趟 46 − 21）。
const STATES_BEFORE_A_DEVICE_HOLDS_TWO_RECORDS_OF_ONE_INSTANCE: u64 = 29;
/// 重放在所选根之上施加记录的状态：暖机第二次与 A 各 3 个（记录落了一份或两份、根没落）。暖机第一次那 3 个不算：记录是实例 1 的，
/// 所选根是 mkfs 的实例 0 种子根，前缀只接同实例的记录，一条都不施加。快档与全量相同（全量多出来的 A 单元段状态上 A 的记录一份都没落）。
/// 对得上同一份日志的 `I-4.1=6/0`，与 oracle 那一半的 `verification_ran_states` 6 是同一批状态（推的：恢复施加了记录才跑验证）。
const STATES_WHERE_THE_REPLAY_APPLIES_A_RECORD: u64 = 6;
/// 每块盘都只剩一槽可择、而且是 mkfs 写下的世代号 1 的状态：取号第一轮那一段两块盘的写都撕开的那 1 个（另一槽还是 mkfs 写的 1）。
/// 取号第二轮那一段两块盘都撕开时，每块盘剩下的是第一轮写的世代号 2 那一槽，照判；之后每一段里每块盘都已有一槽世代号 ≥ 2。
/// 快档与全量相同。快档 I-7.16 = 54 − 1 = 53（取号写一轮时 46 − 1 = 45，对得上同一份日志的 `I-7.16=45/0`）。
const STATES_WHERE_EVERY_DEVICE_HAS_ONE_CHOOSABLE_SLOT_WRITTEN_BY_MKFS: u64 = 1;

fn prepare(tag: &str) -> Prepared {
    let pool = build_pool(tag);
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2],
        "取号写两轮、轮后各一道池屏障，各成一段；C577 之后暖机第二次的系统配置槽轮换过了发布末尾那道屏障，不再与 A 的 24 个单元写同段"
    );
    assert_eq!(
        writes.len(),
        43,
        "取号两轮各 2 + 暖机两次各 5 + A 29（24 个单元写、两份记录、根槽、两块盘的系统配置槽轮换）"
    );
    let root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("写流里有根槽那一条");
    let transaction_publish = TransactionPublishInRecordedStream::find_in(&writes);
    assert_eq!(
        transaction_publish.root_write_index, root_index,
        "被判的那条根槽写就是新池新建文件那次发布的根"
    );
    Prepared {
        base,
        writes,
        segments,
        root_index,
        transaction_publish,
    }
}

/// 全量：[`FULL_STATES`] 个状态。跑得慢（每个状态两遍恢复 + checker + 记录核对器），门禁 checker-tier-crash-point-replay 在 release 下跑它；平时 `cargo test` 跳过。
/// 判过的状态都在判定库里（崩溃枚举用例那一类，`crash_state_enumeration` 的两段：流先录进库，判段从库读回判，被杀之后下一趟接着判没判过的）；
/// I-3.10 那一类的逐状态计数记在观察者计数里，按块存进库、判段重建。双机（`scripts/crash/crash-point-replay-shard-run.sh` 经崩溃放量的双机驱动）：
/// 两台各判自己那一份，那几趟不打计数行、不判下面的断言；导入第二台的库之后再跑的那一趟拿整条流的计数照下面逐项判、逐字打同样的行。
#[test]
#[ignore = "全量六千七百多万个状态，门禁 checker-tier-crash-point-replay （.claude/gate.d/checker-tier-crash-point-replay.sh）在 release 下跑，带断点续跑"]
fn crash_point_replay_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations() {
    let prepared = prepare("crash-point-replay-full");
    assert_eq!(
        closed_form_state_count(&prepared.segments),
        FULL_STATES_WITH_TWO_STATES_PER_WRITE,
        "每次写只取两态时的闭式（E142 产物的 closed_form）"
    );
    // 计数行的 closed_form 报枚举域的闭式（原地覆写三态），checker-tier-crash-point-replay 认 exhaustive=true 就是枚举到的恰好这么多。
    let closed_form = crash_state_count_with_torn_in_place_overwrites(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let Some((tally, read_set_counts)) =
        enumerate_counting_allocation_generation_read_sets(&prepared, "full", &full_expansion)
    else {
        return;
    };
    println!(
        "CRASH_POINT_REPLAY states={} closed_form={closed_form} violations={} root_persisted_states={} no_file={} file_read={} failed={} journal_differing={} exhaustive={} states_by_publish=[{}]",
        tally.states,
        tally.violations,
        tally.root_persisted_states,
        tally.no_file_states,
        tally.file_read_states,
        tally.failed_states,
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
    // A 那一段的单元写只落一部分的状态上 A 的记录与根都没落，恢复落在暖机 txg 2 的根上、没有文件（实六推的，checker-tier-crash-point-replay 全量核）。
    // 系统配置槽撕裂的状态（第三态）与那次写没持久的同一状态恢复得一样：撕裂的那一槽解不开，择的是同一块盘上另一槽，
    // 而没持久时那一槽里是更旧的一代、择的也是另一槽（推的，补第三态之后 checker-tier-crash-point-replay 全量核）；它们不带待施加的记录，journal 承重、验证那几格不变。
    assert_eq!(
        oracle_counts(&tally),
        OracleCounts {
            states: FULL_STATES,
            violations: 0,
            root_persisted_states: ROOT_PERSISTED_STATES,
            no_file_states: FULL_STATES - FILE_READ_STATES,
            file_read_states: FILE_READ_STATES,
            failed_states: 0,
            journal_differing_states: JOURNAL_DIFFERING_STATES,
            first_violation: None,
        }
    );
    assert_eq!(closed_form, FULL_STATES, "全量：枚举到的状态数等于闭式");
    // 按发布分（里程碑「覆盖写、释放、回退与复用」验收「每次发布各多少」那一条的口径，新池新建文件流同样报）：暖机 txg 1 是取号两段各 8 + 3 + 1 = 20，
    // 暖机 txg 2 是 8 + 3 + 1 = 12（暖机第一次的轮换段 3² − 1），
    // A 是暖机第二次的轮换段 8 + 单元段 2²⁴ − 1 = 16777215 + 记录段 3 + 根槽段 1 = 16777227（C577 之前轮换与单元同段、
    // 3² · 2²⁴ − 1 + 3 + 1 = 150994947），A 的系统配置槽轮换 8，再加全部持久那一个。
    assert_eq!(
        tally.states_by_publish_text(),
        "instance1_txg1=20 instance1_txg2=12 instance1_txg3=16777227 after_the_last_root=8 every_write_persisted=1"
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

/// 两遍恢复落到的根、结局码或失败点不同的状态数（`crash_judge_conclusions::recovery_passes_differ`）：A 那一段的记录落了、根槽没落的 3 个状态
/// 看 journal 那一遍读到文件、不看的那一遍没有；A 的根槽落了而系统配置槽没轮换的 3 个状态两遍都读到文件、看 journal 那一遍落到 A 的根、
/// 不看的那一遍落到暖机 txg 2 的根（只比结局时这 3 个不算，钉 3）。快档与全量同一个数（少展开的那几个状态两遍落到同一个根）。
const JOURNAL_DIFFERING_STATES: u64 = 6;

/// 平时跑的那一份（甲二快档，用户 2026-09-26 定）：每段原地写各取它的几态、任意组合 × 单元写全不落或全落——A 的 24 个单元写那一段
/// 只取全不落（1 个），其余十一段全展开——[`QUICK_TIER_STATES`] 个状态；报出来的是「不是全量」。
#[test]
fn crash_point_replay_quick_tier_matches_the_full_tally_shape() {
    let prepared = prepare("crash-point-replay-partial");
    // 交回 None 是这一趟没判：只录那一趟、只判自己那一份的分片，或配置开了 GPU 而这份二进制没编进 gpu 特性
    // （引擎开跑时已打本次未跑那一行，判归带特性那一趟）；这条快档用例的计数等判完全部块的那一趟
    let Some((tally, read_set_counts)) = enumerate_counting_allocation_generation_read_sets(
        &prepared,
        "quick_tier",
        &quick_tier_expansion,
    ) else {
        println!("快档用例 crash_point_replay_quick_tier_matches_the_full_tally_shape：这一趟没判、不比计数（只录、分片或判归带 gpu 特性那一趟）");
        return;
    };
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
            journal_differing_states: JOURNAL_DIFFERING_STATES,
            first_violation: None,
        },
        "少展开的只有 A 那一段单元写只落一部分的组合：文件在 / 根已持久 / journal 承重的状态数都与全量相同"
    );
    assert_eq!(
        tally.states_by_publish_text(),
        "instance1_txg1=20 instance1_txg2=12 instance1_txg3=13 after_the_last_root=8 every_write_persisted=1",
        "甲二按发布分：少的只在 A 那一格（暖机第二次的轮换段 8 + A 的单元段 1 + 记录段 3 + 根槽段 1）"
    );
    assert_eq!(
        crash_state_count_with_torn_in_place_overwrites(
            &prepared.base,
            &prepared.writes,
            &prepared.segments,
            &quick_tier_expansion
        ),
        QUICK_TIER_STATES,
        "甲二按闭式数"
    );
    assert_eq!(
        crash_state_count(&prepared.writes, &prepared.segments, &quick_tier_expansion),
        29,
        "每次写只取两态时甲二的闭式（补第三态之前的数）：1 + 5 × 3 + 1 + 3 × 3 + 3 × 1（取号写一轮时 26）"
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

/// 一张盘上的池级 checker 走两段（用户 2026-10-13 定「能走两段的都走两段」）：那张盘作一张给定状态表（基线就是那张盘、写表空、一个空持久集合）
/// 交崩溃点重放的给定状态引擎（`crash_amplification::GivenStatesEngine`），录入段写进给定状态表那一类的判定库，判段从库里读回判
/// （配了 GPU 走 GPU 判器，没配由 CPU 判器判），判过的复用。只录那一趟、表归别的机器判的那一趟交 `None`（用例当场返回）。
///
/// # Panics
/// 引擎起不了、判不了这张表（不退回别处判）；GPU 判不了这张盘（第三轮正文 H6 判红）。
fn pool_checker_conclusion_through_the_given_states_engine(
    table_name: &str,
    image: &MemoryPool,
) -> Option<GivenStateConclusion> {
    let engine = {
        let _one_test_at_a_time = one_test_changes_the_environment_at_a_time();
        GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: "crash_enumeration_new_pool_file_creation_stream::fixed".to_string(),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    };
    let judged = engine
        .judge_the_given_states(&GivenStatesOfATable {
            table_name: table_name.to_string(),
            base: image,
            writes: &[],
            segments: &[],
            seams: &[],
            persisted_sets: vec![Vec::new()],
        })
        .unwrap_or_else(|reason| {
            panic!("给定状态表 {table_name} 判不了（不退回别处判）：{reason}")
        });
    assert_eq!(
        engine.finish().red_items(),
        Vec::<String>::new(),
        "GPU 判不了的盘照第三轮正文 H6 判红"
    );
    match judged.source {
        GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => {}
        GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost
        | GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => return None,
    }
    let [conclusion] = <[GivenStateConclusion; 1]>::try_from(judged.states)
        .unwrap_or_else(|states| panic!("一个状态一份结论，交回 {} 份", states.len()));
    Some(conclusion)
}

/// checker 在写完新池新建文件的镜像上：每条第一版不变量的判定（判到位与违例位；违例带细节的是这一趟 CPU 判的，GPU 判的与从库里读回的不带）。
#[test]
fn checker_report_on_the_final_image() {
    let pool = build_pool("checker-final");
    let Some(conclusion) =
        pool_checker_conclusion_through_the_given_states_engine("final-image", &pool.memory_pool())
    else {
        return;
    };
    let violations = conclusion.pool_checker_violations();
    for (position, invariant) in singlefs_checker::image::IMPLEMENTED_INVARIANTS
        .iter()
        .enumerate()
    {
        let bit = 1u128 << position;
        let verdict = if conclusion.words.pool_checker_violated_bits & bit != 0 {
            violations
                .iter()
                .find(|(violated, _)| violated == invariant)
                .map_or_else(String::new, |(_, detail)| format!("Violated({detail:?})"))
        } else if conclusion.words.pool_checker_judged_bits & bit != 0 {
            "Holds".to_string()
        } else {
            "NotApplicable or PartiallyJudged".to_string()
        };
        println!("FINAL {invariant} {verdict}");
    }
}

/// 阳性对照（靶向，不是 E142 那条单盘无屏障臂）：根槽已持久而某个单元两份都没持久——这正是段与段之间少一道屏障会放进来的那类状态。
/// oracle 对 12 个单元逐个都要判红；全部持久那一个判绿；根槽已持久而 journal 记录两份都没持久不算违例（根自己带着全部字段）。
#[test]
fn positive_control_root_persisted_without_each_unit_is_caught_by_the_oracle() {
    let prepared = prepare("crash-point-replay-control");
    let all_persisted = vec![true; prepared.writes.len()];
    let mut tally = CrashPointReplayTally::default();
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
        let mut control_tally = CrashPointReplayTally::default();
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
    let mut control_tally = CrashPointReplayTally::default();
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
/// 摘掉新池新建文件里「journal 记录 → 根槽」那道池屏障（录制器按设备记屏障，两块盘各一步，两步都摘），两份记录、根槽与之后的
/// 两次系统配置槽轮换写同段（段按设备记屏障：根槽 FUA 只放行它自己那块盘，另一块盘上的记录没放行，这一段开到轮换之后那道屏障才关）；
/// 枚举里出现根在案而记录一份都不在的状态。oracle 在这里不红。只摘盘 1 那一步的判别力在
/// `crash_segments_per_device_and_torn_in_place_overwrites.rs`（那里同样是 `[…, 24, 5]`、同样按 `<= 5` 展开）。
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
        vec![2, 2, 2, 1, 2, 2, 1, 2, 24, 5],
        "记录两份、根槽与 A 的两次系统配置槽轮换写并成一段（取号两轮各成一段；C577 之后暖机第二次的轮换 2 写自成一段，不与 A 的 24 个单元写同段）"
    );
    let root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("根槽");
    // A 的 24 个单元写那一段不展开（只以整段持久进入后面的状态），其余每段任意子集；最后那一段 5 个写，选择器放到 5 才展开得到它
    // （留在 3 时这一段只以整段持久进来，根在案而记录不在的状态一个都摆不出来）。
    let Some(tally) = enumerate_crash_states_selecting(
        CrashPointReplayItem {
            name: "new_pool_file_creation::barrier_removed_before_the_root_slot",
            setting_file: THIS_FILE,
        },
        &base,
        &writes,
        &segments,
        root_index,
        &file_content(),
        &|_segment_index, segment| segment.len() <= 5,
    )
    .judged() else {
        return;
    };
    assert_eq!(tally.violations, 0, "oracle 不红：根记录自己带着全部字段");
    // 五个写那一段（记录 2、根槽 1、系统配置槽轮换 2）里根槽落了、两份记录都没落的状态：两次轮换写是原地覆写、各取三态（没落、落了、撕开），
    // 3² = 9 个，都是真子集（记录没落）。记录核对器的「根在案而记录不在」只看盘上的根槽与记录写，不看系统配置槽（推的，checker 档没跑）。
    assert_eq!(
        tally.record_root_without_record, 9,
        "五个写那一段的真子集里根槽在、两份记录都不在的是两次轮换写各三态的 3² 个"
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

/// 观察者另读的输入的摘要要真算：全零就是没算（另读输入不同的两趟会共用看过的块）。
fn extra_inputs_digest_that_was_computed(
    digest: [u8; singlefs_harness::sha256::DIGEST_BYTES],
) -> [u8; singlefs_harness::sha256::DIGEST_BYTES] {
    assert_ne!(
        digest,
        [0; singlefs_harness::sha256::DIGEST_BYTES],
        "观察者另读的输入的摘要真算（全零就是没算：另读输入不同的两趟会共用看过的块）"
    );
    digest
}
