//! 崩溃放量流水线（里程碑二「增补 4」第一、四、十一项形态）：
//!
//! 1. **崩溃点与路径**：一条测试流程是从 mkfs 出发、一个崩溃点接一个崩溃点走下来的路径；每段归含它最后一次写的那个点，上游不看下游。每个点的身份
//!    `<仓内路径>::<崩溃点名称>::<哈希串>`（[`crate::crash_identity`]）只作索引：进路径索引表与覆盖报告，不进块键（键里不许有
//!    路径、名字、种子）。几条流程共用前缀，全部点连成一棵树（「这就是个 B 树的问题」）：
//!    路径编号是从根到这个点、每个点取输入键（点名、写表、父镜像的摘要）前 16 个十六进制、用 `/` 逐段接起来的串：
//!    前面几段相同就是同一段枝干，一棵子树就是一段连续的键。哈希串（复用键）只由流程定：这一步写下的字节与它起步的镜像；
//!    按 COW 的性质，一个点的输入不变，它的崩溃状态就不变。判法不进任何键：结论按（块键，结论种类，结论版本）存，结论版本由结论种类与判器引到的被检测产品代码定
//!    （[`ConclusionVersion::of_the_crash_image_judgement`] 一族；判器、观察者与录制管线的代码都不进，改判器不作废已判的块，判法改了对不对靠对拍
//!    与复用哨兵抓），CPU 判与 GPU 判同一把；同一块上几个观察者的结论按各自的判定项标识分开（[`ObserverVersions`]）；
//!    记着红的块不复用、下一趟现判，判完照新结论换上（「记红、撤绿、下趟现判」）；每一趟真跑抽复用哨兵（[`plan_the_reuse_sentinels`]）：
//!    候选是键相同、库里有不红的判器结论、挂在那个点上的每个观察者也都有不红结论的块，照种子抽几块当哨兵、再抽一块强制现判（判器结论与观察者结论
//!    一起现判、一起比）；哨兵被现判、强制现判那一块仍复用、或它现判出的与库里那一份不同，整趟红，红的那一句点名块键与种子。
//!    种子取 `SINGLEFS_REUSE_SENTINEL_SEED`（[`REUSE_SENTINEL_SEED_ENVIRONMENT_VARIABLE`]，十进制非负整数），没设就现取一个；抽几块取
//!    `SINGLEFS_REUSE_SENTINEL_COUNT`（[`REUSE_SENTINEL_COUNT_ENVIRONMENT_VARIABLE`]），没设取 [`DEFAULT_REUSE_SENTINEL_COUNT`]；
//!    这一趟的种子打在 [`REUSE_SENTINELS_LINE_PREFIX`] 那一行，带同一个种子重跑抽到同一组。两个变量都不在统一任务服务的白名单里：
//!    复现某一趟，把种子写在被交的命令前面，`task-server/task submit -- env SINGLEFS_REUSE_SENTINEL_SEED=<种子> <原命令…>`。
//!    强制现判那一块连同块里各张盘上的池级 checker 判定一起现判（不取库里的）；它上面观察者要读的录入段那一行不在库里的，回录入段只给它重录
//!    （[`record_the_observers_of_the_forced_block_again`]），不丢掉再抽（两条都是）。
//!    判器判一个点的状态时另读的流程输入（这个点之前的写表、到这个点为止够得着的版本表、没做完的那次发布的根槽写；被判的根槽写不在里面，
//!    的 M2-2 甲）
//!    不进复用键与身份（树不动），进这个点的计划哈希、块键（`JudgeInputsOfTheFlow`）：只差这些的两条流不互相复用判定。
//!    按发布分的态数要往下游找的点（段里与点里后面的段都没有根槽写），那几段归的发布（下游第一次根槽写，或后面没有根槽写）同样只进
//!    计划哈希（`publishes_looked_up_downstream_of_the_point`「乙」）：块上存的计数向量跟着下游变，下游不同的两条流不共享这几块。
//! 2. **先录后核**：录入把每个点的状态按块（点内序号区间）登进库的录入表；核对按批从库里取还没有判定的块，
//!    在 CPU 上（或 CPU + GPU：单元两道校验和上 GPU，与 CPU 对拍）判完、原子地写回判定块与违例；门禁一次读完。
//! 3. **对账**：一轮收尾，录入表里有、判定块表里没有的块就是没核完的（卡掉线、进程被杀），重新分给还在的一方补跑；
//!    领块只在内存里，库里不记谁领了，不会有块被一张死卡占着。
//! 4. **分工与跑法**：块按权重分给几方（一方一台机器的 CPU，或一张卡，权重取空闲显存），各方都直接写 prune-server 里那一份库
//!    （不各写各的再取回、导入）；跑法由配置的两个开关定：`ENABLE_ACROSS_MACHINES`、`ENABLE_GPU`。
//!    配了 GPU 的判段（崩溃放量与崩溃枚举用例）不在自己进程里持卡判：把要判的块写成待判行、等 GPU 批处理服务判完
//!    （`judge_my_blocks_through_the_pending_rows` 那一节）；崩溃放量第 ② 段核对同样写核对待判行、等服务核完
//!    （`verify_my_blocks_through_the_facts_verifier_pending_rows` 那一节）；登记、写行与等的时候都不问服务收不收（待判行不带判器的版本，服务只照行的数据判），
//!    判段的库在临时 prune-server 里时直接在真服务里登记那份临时库（[`temporary_library_of_the_judging_stage_registered_with_the_real_service`]），
//!    服务读不了就点名跳过，判段等不到的由统一任务服务照卡住停；只排派活计划那一趟只数两段要判、要核的块与状态，
//!    不取卡、不试判（[`placement_line_of_the_pending_blocks`]）。下面第 5 条里第 ① 段在卡上判的那一套只给直接调 `judge_my_blocks_on_gpu_cards` 的对拍用例用。
//! 5. **显存账**（`singlefs_verification::gpu_memory_ledger`，只防这台机器的显存 OOM、不压并发；只记这个进程借的、
//!    每次借现读卡上此刻的显存：岔路 G1「二」，口径在那份文件头）：带 `gpu` 特性的每个进程取卡都照这个进程里 GPU 用例共用的卡池
//!    （[`gpu_cards_in_use`]，没起过当场起；[`GpuCardsSource::TheCardPoolOfThisProcess`]），不登记也不起设备，借到哪张卡才在哪张上起：
//!    同一个进程里只有一套显存账，经引擎起步的判与直接借卡的 GPU 用例互相看得见；
//!    第 ① 段在卡上判的那一套（只有直接调 `judge_my_blocks_on_gpu_cards` 的对拍用例走，卡由用例自己起好交进来、不进账）每张卡装几个状态
//!    照起设备那一刻的额度算；每张卡此刻都带不了一个状态而至少一张空着时带得了，照对拍用例那一路打「本次未跑」那一行
//!    （同一个记号，原因写显存不够）、一块都不判、交回错；空着时也带不了才判红（`singlefs_verification::gpu_memory_ledger::decide_scratch_before_booking`）。
//!    第 ① 段由 CPU 判时的单元校验和对拍（`gpu_unit_checks`）在卡上建的缓冲同样进账：照卡的先后借第一张
//!    此刻放得下的，用借来的那张卡算，借不到报本次未跑（判过的块在库里）。
//!    驱动的上下文与编好的管线算在每张卡第一次借时的驱动那一份里，不另记。
//!    账只记这个进程借的：两个进程同一刻读到同一份空闲、一起借超了，由运行时兜底：第 ① 段卡上建判器、判一块时 wgpu 报 Out of Memory，
//!    这张卡停下、这一块交回给别的卡，判不完才报本次未跑（原因写显存不够，`singlefs_verification::gpu_memory_ledger::reason_for_running_out_of_gpu_memory`）；
//!    卡池在借到的卡上起设备与编单元校验和管线时同样。
//!    判器内核在另起的线程上编，那里的 Out of Memory 不进卡线程的错误范围：每个设备上装了接 wgpu 没接住的错误的回调，Out of Memory
//!    记在设备上（`crate::gpu_unit_checks::DeviceConditionOfACard`），卡线程上的活 panic 了就照它认是不是显存不够
//!    （`decide_the_work_on_the_card`），按这段活认、不按整台设备认（别的线程一次 Out of Memory 不把这条线程上真正的 panic
//!    说成显存不够）；设备因为 Out of Memory 丢了，卡池下一次借这张卡时重起设备（驱动那一份在账上重新登记，放不下就报本次未跑）。
//!    第 ① 段在几张卡上判时，一张卡建判器或判一块时报了 Out of Memory，只停那一张：它手里那一块交回队里，别的卡照判
//!    （P2）；剩下的块别的卡判完了这一趟照常交回。别的卡都已收工、还有交回的块没判时，收尾那一步
//!    再起一轮，只用没报过的卡（重新装状态）判剩下的块，判完的块照块序都入库（
//!    K2-1 第 5 条：状态全装在第一张卡时也起第二张）；报过的卡之外没有卡能判才报本次未跑。被叫停时卡照样领判完的块在等的那几块（交回的），
//!    判完的块照块序都入库再照叫停收尾，没判的留给下一趟；照叫停收尾而这一趟有卡报过
//!    Out of Memory，头一句照「本次未跑」那一行的形态打出来。叫停是叫停文件在不在，
//!    一趟里见过一次就记住（`singlefs_verification::gpu_memory_ledger::StopRequestRememberedInThisRun`：
//!    外层循环、卡线程、一轮收尾、整趟收尾都问它，文件中途被删也不各读各的）。卡线程那一圈、起下一张卡、活交回那一刻记设备与按块序写库、
//!    几轮之间带着的与怎么走在 `gpu_memory_ledger`（`singlefs_verification::gpu_memory_ledger::judge_blocks_on_one_card`、`next_card_to_start`、
//!    `work_caught_noting_the_device_when_it_returned`、`BlocksStoredInOrder`、`RoundsOfTheCards`），不要卡就测得到。
//!    单元校验和对拍时一块的几段在同一个设备上并发，认一段是不是显存不够之前等同块的几段都收完（攻方：一段因设备丢了先失败，
//!    同块另一段范围里接到的 Out of Memory 还没记进设备时不判红）。
//!    卡由调用方自己起好交进来的（[`GpuCardsSource::DevicesStartedForThisRun`]）不进账：给定状态表与观察者交回的盘那两路照卡的来路分两支，
//!    卡池那一支照上面借，这一支在卡上建判器之前按显存排队（`singlefs_verification::gpu_memory_ledger::CardMemoryQueue`，
//!    [`judge_on_a_card_started_for_this_run_queued_by_memory`]）；这两路的卡只经 [`gpu_cards_in_use`] 取，走的是卡池那一支。
//!    直接调 `judge_my_blocks_on_gpu_cards` 的对拍用例不走这一条，卡由用例自己给、不进账。

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::path::{Path, PathBuf};
#[cfg(feature = "gpu")]
use std::time::Instant;

#[cfg(feature = "gpu")]
use crate::gpu_memory_ledger::{
    decide_scratch_before_booking, gpu_not_run_line, reason_for_a_booking_without_room_now,
    reason_for_running_out_of_gpu_memory, scratch_booking_refusal_outcome, BookingRefusal,
    NeedLargerThanEveryCard, ScratchBeforeBooking, ScratchBookingRefusalOutcome,
};
#[cfg(feature = "gpu")]
use crate::gpu_memory_ledger::{
    judge_blocks_on_one_card, next_card_to_start, work_caught_noting_the_device_when_it_returned,
    AfterARoundOfTheCards, BlockHandedToTheStore, BlockJudgedOnACard, BlocksForTheCards,
    BlocksStoredInOrder, CardLeftTheBlocks, RoundsOfTheCards, StopOfTheCards,
};
use crate::gpu_memory_ledger::{CardMemoryQueue, StopRequestRememberedInThisRun};
#[cfg(feature = "gpu")]
use crate::gpu_memory_ledger::{WorkEndingTogether, WorksEndingTogetherOnADevice};
use crate::heavy_threads::{
    own_variable_or_default_heavy_threads, THREAD_CAP_ENVIRONMENT_VARIABLE,
};
use crate::hexadecimal::hexadecimal_text;
use crate::memory_pool::{
    root_identity_of_write, root_identity_written_by, CrashImage, MemoryPool, PublishedVersion,
    PublishedVersionWithoutAFile, RetainedWrite, WrittenContents,
};
use crate::numeric_environment_variable::{
    number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
};
use crate::segments::StepKind;
use crate::sha256::{sha256_digest, DIGEST_BYTES};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome, RecoveryReport};
use singlefs_core::root_record::ROOT_MAGIC;

use singlefs_checker::image::IMPLEMENTED_INVARIANTS;

use crate::crash::{
    crash_state_ranges_by_segment_with_slices, no_segment_slices, publish_of_each_segment,
    receive_until_the_senders_are_gone_with_wall_clock_lines, CrashPointReplayObserverCounts,
    CrashPointReplayPublishOfState, CrashPointReplaySegmentExpansion, CrashPointReplaySegmentSlice,
    CrashPointReplayVersionTable, ReceivedOrWallClockLineDue, StateCount1280Bits,
    TearableInPlaceOverwrites, Unsigned256Bits, WallClockProgressLines, STATE_COUNT_BYTES,
    STATE_COUNT_LIMBS,
};
use crate::crash_facts::{
    extract_flow_facts_with_slices, segment_expansion_facts,
    verify_states_from_facts_by_the_segment_expansions, FlowFacts, SegmentExpansionFact,
};
use crate::crash_given_states::{
    count_vector_of_the_given_states, judge_the_given_states_on_the_cpu,
    AbandonedRootNamedByThePoolChecker, ClaimedVersionOfAGivenState, ClaimedVersionsOfTheStates,
    GivenStateConclusion, GivenStatesConclusionsOfATable, GivenStatesConclusionsSource,
    GivenStatesJudge, GivenStatesOfATable, GivenStatesOfATableWithItsColumns,
};
use crate::crash_identity::{
    content_digest_text, observer_product_code_key_in, step_write_table_with_barriers,
    ConclusionKind, ConclusionVersion, CoverageReport, CoverageState, JudgingUnits,
    JudgingUnitsKey, JudgingUnitsKeyRefused, NodeLabel, ObserverJudgingUnitsKeyTable, ObserverName,
    ReuseKey, SectorImage, StepAttributes,
};
use crate::crash_judge_conclusions::{
    cross_check_words_of_a_stored_given_state, encoded_verdict_of_the_red_items,
    recovery_words_of_the_cross_check_words, red_items_of_a_given_state,
    red_items_of_the_encoded_verdict, self_verified_system_configuration_slots_of_each_device,
    CrossCheckWords, LandedRootWords, RunOfStatesWithTheSameSelfVerifiedSlots,
    SelfVerifiedSystemConfigurationSlotsHandedBackForABlock,
    SelfVerifiedSystemConfigurationSlotsOfTheStates, StateCountsOfABlock, LANDED_ROOT_WORDS,
    RECOVERY_OUTCOME_CODE_FAILED, RECOVERY_OUTCOME_CODE_FILE_READ, RECOVERY_OUTCOME_CODE_NO_FILE,
    SCAN_REBUILD_OUTCOME_NOT_REACHED,
};
use crate::crash_judge_cpu_blocks::{
    judge_a_piece_of_a_block_on_the_cpu, CpuJudgedBlock, RED_TEXTS_KEPT,
};
/// GPU 结论换红项的换法在判器一侧（`crate::crash_judge_gpu`：改结论的几处搬进判器一侧），
/// 这里照旧转出去：对拍用例经这条路径引。GPU 结论换恢复摘要（`gpu_recovery_words`）交的是这里的 [`RecoveryRootAndOutcomesOfState`]，
/// 写在这一侧：判器一侧不经它的返回类型够到流水线（判器闭包不许有流水线的单元，`crash_identity` 的
/// `the_judge_groups_reach_no_pipeline_unit_and_the_cpu_groups_no_card_unit`）。
#[cfg(feature = "gpu")]
pub use crate::crash_judge_gpu::gpu_verdict_items;
#[cfg(feature = "gpu")]
use crate::crash_judge_gpu::{
    pool_checker_judged_bits, pool_checker_violated_bits, recovery_words_of_the_gpu_verdict,
    self_verified_system_configuration_slots_of_each_device_of_the_gpu_verdict,
};
#[cfg(feature = "gpu")]
use crate::crash_judge_gpu::{
    CONSULTED_PASS_LANDED_CHECKPOINT_TXG_HIGH_WORD, CONSULTED_PASS_LANDED_CHECKPOINT_TXG_LOW_WORD,
    CONSULTED_PASS_LANDED_INSTANCE_WORD, CONSULTED_PASS_LANDED_MARKER_WORD,
    CONSULTED_PASS_OUTCOME_WORD, IGNORED_PASS_OUTCOME_WORD,
};
use crate::crash_library::CRASH_LIBRARY_RUN_KIND_ENVIRONMENT_VARIABLE;
pub use crate::crash_library::{
    LibraryOpenInThisProcess, LibraryTakenAtEachReadOrWrite, RunStage,
    RUN_STAGE_ENVIRONMENT_VARIABLE,
};
use crate::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
pub use crate::verdict_store::{tokyo_date_text_of, RunKindOfTheJudgement};
use crate::verdict_store::{
    BlockCountVector, BlockStartStateIndex, ConclusionBlockRecord, EncodedVerdictVector,
    EnumerationPlanHash, InvariantStateCounts, JudgeOfTheProgress, JudgingDevice,
    JudgingUnitsFingerprint, NodeInputDigest, ObserverReadSetWitnessRow, RecordingKey,
    RecoveryWordsOfState, SlicedSegmentProgress, SlicedSegmentProgressKey, StateOffsetInBlock,
    StoredConclusionBlock, StoredViolation, VerdictBlock, VerdictBlockKey, VerdictStore,
    VerdictStoreError, VerdictVectorNumber,
};
// 只有流水线（录入段、复用哨兵挑候选）用的那一种结论块问法，单独一行引进来：观察者结论拼块那一组的闭包连着上面那一行
use crate::verdict_store::ConclusionBlockInTheLibrary;
use singlefs_checker::walk::{
    ContainerCapacityKnobs, LineageContainerCapacityKnob, RetirementContainerCapacityKnob,
    TombstoneContainerCapacityKnob,
};

mod block_ranges_and_parts;
mod block_recording_and_storage;
mod conclusion_versions_and_judging_units;
mod conclusions_read_at_the_end;
mod cpu_checking;
mod crash_point_plan;
mod crash_point_recorder;
mod facts_verification;
mod facts_verifier_pending_rows;
mod given_states_engine;
mod gpu_card_judging;
mod gpu_pending_rows_and_packed_tables;
mod gpu_verification_server;
mod images_after_the_observers;
mod observer_definitions;
mod observer_judging;
mod observer_recording;
mod observer_records;
mod read_set_witnesses;
mod recorded_flow;
mod registered_segments_and_slices;
mod reuse_sentinels;
mod run_from_the_environment;
mod run_modes_and_cards;
#[cfg(test)]
mod shared_test_fixtures;
#[cfg(test)]
mod source_reading_for_the_scans;

pub use block_ranges_and_parts::*;
pub use block_recording_and_storage::*;
pub use conclusion_versions_and_judging_units::*;
pub use conclusions_read_at_the_end::*;
pub use cpu_checking::*;
pub use crash_point_plan::*;
pub use crash_point_recorder::*;
pub use facts_verification::*;
pub use facts_verifier_pending_rows::*;
pub use given_states_engine::*;
pub use gpu_card_judging::*;
pub use gpu_pending_rows_and_packed_tables::*;
pub use gpu_verification_server::*;
pub(crate) use images_after_the_observers::*;
pub use observer_definitions::*;
pub(crate) use observer_judging::*;
pub(crate) use observer_recording::*;
use observer_records::*;
use read_set_witnesses::*;
pub use recorded_flow::*;
pub use registered_segments_and_slices::*;
pub use reuse_sentinels::*;
pub use run_from_the_environment::*;
pub use run_modes_and_cards::*;
#[cfg(test)]
use shared_test_fixtures::*;
#[cfg(test)]
use source_reading_for_the_scans::*;

#[cfg(test)]
mod tests {
    use super::*;

    /// 扫源码的用例照它读族里源码的文件表（[`CRASH_AMPLIFICATION_FAMILY_FILES`]）就是根起逐层 `mod` 声明到的全部文件，每一份归的那一种
    /// （编进库、只给测试编）也与声明的相同（[`submodule_files_declared_in`]）：族里新加、删掉一份子模块而表没跟上，或一份归错了，扫源码的用例
    /// 就少扫、多扫一份，这条红。表照路径的字节序排、不重，根在最前（扫源码的用例照这个次序一份接一份读）。
    #[test]
    fn the_family_table_holds_every_file_the_module_declarations_reach_from_the_root() {
        let table: Vec<(&str, RoleInTheFamily)> = CRASH_AMPLIFICATION_FAMILY_FILES
            .iter()
            .map(|(path, role, _source)| (*path, *role))
            .collect();
        assert_eq!(
            table.first(),
            Some(&(ROOT_OF_THE_FAMILY, RoleInTheFamily::CompiledIntoTheLibrary)),
            "根在最前、编进库"
        );
        assert!(
            table.windows(2).all(|pair| pair[0].0 < pair[1].0),
            "表照路径的字节序排、不重：{table:#?}"
        );
        let in_the_table: BTreeSet<(String, RoleInTheFamily)> = table
            .iter()
            .map(|(path, role)| ((*path).to_owned(), *role))
            .collect();
        let declared: BTreeSet<(String, RoleInTheFamily)> = std::iter::once((
            ROOT_OF_THE_FAMILY.to_owned(),
            RoleInTheFamily::CompiledIntoTheLibrary,
        ))
        .chain(
            CRASH_AMPLIFICATION_FAMILY_FILES
                .iter()
                .flat_map(|(path, role, source)| submodule_files_declared_in(path, *role, source)),
        )
        .collect();
        assert_eq!(
            in_the_table, declared,
            "文件表与根起逐层 mod 声明到的文件（连同归的那一种）不一样"
        );
    }

    /// K5：崩溃放量模块族编进库的那几份（[`library_files_of_the_family`]，各去掉测试部分）里每一处按名字读环境变量
    /// （`env::var("…")`、`env::var_os("…")`），除守卫留着的那几个
    /// （[`ENVIRONMENT_VARIABLES_A_RUN_READS_THAT_THE_GUARD_KEEPS`]）之外，名字都在 [`ENVIRONMENT_VARIABLES_READ_BY_A_RUN`] 里：
    /// 用例的环境守卫照这张表清、还原，漏一个就会读到调用方留下的值。
    #[test]
    fn every_environment_variable_this_file_reads_by_name_is_in_the_list_a_run_reads() {
        let names_read = environment_variables_read_by_name_in(
            &family_text_outside_the_tests(&library_files_of_the_family()).join("\n"),
        );
        // 多机配置与显卡先后由 `crate::gpu_card_configuration` 读，连 prune-server 哪一处由 `crate::crash_library` 读，这一族按名字读的不少于七处
        assert!(
            names_read.len() >= 7,
            "按名字读的环境变量少得不对，扫法坏了：{names_read:?}"
        );
        assert!(
            names_read
                .iter()
                .any(|name| name == SEGMENTS_REGISTRY_ENVIRONMENT_VARIABLE),
            "按段登记表的路径按名字读：{names_read:?}"
        );
        for name in &names_read {
            if ENVIRONMENT_VARIABLES_A_RUN_READS_THAT_THE_GUARD_KEEPS
                .iter()
                .any(|(kept, _why)| *kept == name.as_str())
            {
                continue;
            }
            assert!(
                ENVIRONMENT_VARIABLES_READ_BY_A_RUN.contains(&name.as_str()),
                "{name} 在崩溃放量模块族编进库的那几份里按名字读，却不在 ENVIRONMENT_VARIABLES_READ_BY_A_RUN 里"
            );
        }
    }

    /// `source`（`crash_library.rs`，或别的一份测试模块写在自己里面的源文件）测试模块之外按名字读的环境变量
    /// （[`environment_variables_read_by_name_in`]）。
    fn environment_variables_read_by_name_outside_the_tests(source: &str) -> Vec<String> {
        environment_variables_read_by_name_in(&text_outside_the_tests_module(source))
    }

    /// `outside_the_tests`（一份源码、或崩溃放量模块族编进库的那几份接起来，测试部分都已去掉；或用例在它上面改过的）里按名字读的环境变量，
    /// 照读法分两轮、各照文本次序：参数是串字面量的（`env::var("…")`、`env::var_os("…")`），与参数是常量的（`env::var(名字)`、
    /// `env::var_os(crate::…::名字)`，名字大写开头，取同一段文本里 `const 名字: &str = "…";` 那个串）。参数是小写开头的局部变量的
    /// （照传进来的名字转读的那几处）不算。
    ///
    /// # Panics
    /// 参数是常量，而同一段文本里没有它恰好一处 `const 名字: &str = "…";`（扫法认不出，不悄悄跳过）。
    fn environment_variables_read_by_name_in(outside_the_tests: &str) -> Vec<String> {
        let mut names_read = Vec::new();
        for opening in ["env::var(", "env::var_os("] {
            for (position, _) in outside_the_tests.match_indices(opening) {
                let argument = outside_the_tests[position + opening.len()..].trim_start();
                if let Some(literal) = argument.strip_prefix('"') {
                    let name = &literal[..literal.find('"').expect("变量名的串收得了尾")];
                    names_read.push(name.to_string());
                    continue;
                }
                let path_end = argument
                    .find(|character: char| {
                        !(character.is_alphanumeric() || character == '_' || character == ':')
                    })
                    .unwrap_or(argument.len());
                let constant_path = &argument[..path_end];
                let (module_path, constant_name) = match constant_path.rsplit_once("::") {
                    Some((module_path, constant_name)) => (Some(module_path), constant_name),
                    None => (None, constant_path),
                };
                if constant_name.starts_with(|character: char| character.is_ascii_uppercase()) {
                    let defining_code = code_outside_the_tests_that_defines_constants_of(
                        module_path,
                        outside_the_tests,
                    );
                    names_read.push(value_of_the_string_constant(&defining_code, constant_name));
                }
            }
        }
        names_read
    }

    /// 按路径 `<module_path>::名字` 读的常量定义在哪一段测试部分之外的文本里：不带路径、或路径是 `self`、`super` 的，就是读它的那一段
    /// （`outside_the_tests_of_the_reading_file`：一份源码，或崩溃放量模块族编进库的那几份接起来）；带 `crate::gpu_card_configuration` 的，
    /// 是那一份源码测试模块之外。
    ///
    /// # Panics
    /// 路径指的是别的模块：扫法不知道那份源码在哪，认不出（不悄悄跳过）。
    fn code_outside_the_tests_that_defines_constants_of(
        module_path: Option<&str>,
        outside_the_tests_of_the_reading_file: &str,
    ) -> String {
        match module_path {
            None | Some("self") | Some("super") => outside_the_tests_of_the_reading_file.to_string(),
            Some("crate::gpu_card_configuration") => {
                text_outside_the_tests_module(include_str!("gpu_card_configuration.rs"))
            }
            Some(other_module_path) => panic!(
                "按 {other_module_path}:: 下的常量读的环境变量：扫法不知道那个模块的源码在哪，认不出"
            ),
        }
    }

    /// 一趟里读、守卫却不清的环境变量，各带为什么：清掉它们用例就跑不起来，它们也不是一趟的旋钮。
    const ENVIRONMENT_VARIABLES_A_RUN_READS_THAT_THE_GUARD_KEEPS: [(&str, &str); 4] = [
        (
            crate::crash_library::PRUNE_SERVER_SOCKET_ENVIRONMENT_VARIABLE,
            "连 prune-server 哪一处：临时服务底下它指临时服务，清掉它产品那一路会落到 XDG_RUNTIME_DIR 下真服务的 socket 上",
        ),
        (
            crate::crash_library::PRUNE_SERVER_TCP_ADDRESS_ENVIRONMENT_VARIABLE,
            "socket 不在时连哪一处：临时服务底下它指临时服务，清掉它会落到多机配置里真服务的地址上",
        ),
        (
            crate::crash_library::PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET_ENVIRONMENT_VARIABLE,
            "临时服务的标记（只有脚本自证的包装设它）：包装底下清掉它，改连接地址的两个变量在产品那一路被拒",
        ),
        (
            crate::gpu_card_configuration::TASK_SERVER_SOURCE_REPOSITORY_ENVIRONMENT_VARIABLE,
            "任务服务交的提交方仓根：判段入口照它取问 GPU 批处理服务的控制命令（与借卡客户端同一个文件）；清掉它控制命令落到从当前目录往上找到的那一份，\
             用例设的假仓里的假控制命令认不出",
        ),
    ];

    /// K5 的另一半：一趟经崩溃判定库（`crash_library.rs`：选段、连 prune-server 哪一处、临时服务的标记）读的环境变量，守卫同样要清、
    /// 要还原。那一份测试模块之外按名字读的（那一份几乎都按常量读），除 [`ENVIRONMENT_VARIABLES_A_RUN_READS_THAT_THE_GUARD_KEEPS`]
    /// 点名的之外，都在 [`ENVIRONMENT_VARIABLES_READ_BY_A_RUN`] 里。
    /// 判别力（`crates/mutations.tsv`）：表里漏了选段的变量，这条红；守卫留着的那张漏了临时服务的标记（两张都不在），这条同样红。
    #[test]
    fn every_environment_variable_the_crash_library_reads_by_name_is_in_the_list_a_run_reads() {
        let names_read =
            environment_variables_read_by_name_outside_the_tests(include_str!("crash_library.rs"));
        assert!(
            names_read
                .iter()
                .any(|name| name == crate::crash_library::RUN_STAGE_ENVIRONMENT_VARIABLE),
            "按常量读的环境变量也认得出（库那一份几乎都按常量读，认不出就什么都没核）：{names_read:?}"
        );
        for name in &names_read {
            if ENVIRONMENT_VARIABLES_A_RUN_READS_THAT_THE_GUARD_KEEPS
                .iter()
                .any(|(kept, _why)| *kept == name.as_str())
            {
                continue;
            }
            assert!(
                ENVIRONMENT_VARIABLES_READ_BY_A_RUN.contains(&name.as_str()),
                "{name} 在 crash_library.rs 里按名字读，却不在 ENVIRONMENT_VARIABLES_READ_BY_A_RUN 里"
            );
        }
    }

    /// 崩溃放量模块族编进库的那几份（[`library_files_of_the_family`]）各去掉测试部分之后的代码行（注释行不算），照文件表的次序一份接一份。
    fn code_lines_outside_the_tests() -> Vec<String> {
        family_text_outside_the_tests(&library_files_of_the_family())
            .iter()
            .flat_map(|outside_the_tests| outside_the_tests.lines())
            .filter(|line| !line.trim_start().starts_with("//"))
            .map(str::to_string)
            .collect()
    }

    /// 判器在这一趟自己起的卡上还占位之前那一次等（`poll_type_before_returning_a_place_in_the_memory_queue`）要真等设备
    /// （`wgpu::PollType::Wait`，带上限）：换成 `Poll` 就不等了，还占位的线程接着往下走、卡上的活没做完。扫崩溃放量模块族编进库的那几份钉住那几行。
    #[test]
    fn the_wait_before_returning_a_place_in_the_memory_queue_is_a_bounded_wait_not_a_poll() {
        let lines = code_lines_outside_the_tests();
        let definition = lines
            .iter()
            .position(|line| {
                line.trim_start()
                    .starts_with("fn poll_type_before_returning_a_place_in_the_memory_queue(")
            })
            .expect("等设备那一次的 poll 方式只由这一个 fn 定");
        let body: Vec<&str> = lines[definition..definition + 6]
            .iter()
            .map(|line| line.trim())
            .collect();
        assert!(
            body.contains(&"wgpu::PollType::Wait {"),
            "还占位之前那一次等要是 wgpu::PollType::Wait：{body:?}"
        );
        assert!(
            body.iter().all(|line| !line.contains("PollType::Poll")),
            "不许换成不等的 Poll：{body:?}"
        );
    }

    /// 叫停文件只在一处读进这一趟记住的那一个叫停
    /// （`StopRequestRememberedInThisRun::new`），外层循环、卡线程、一轮收尾与整趟收尾不再各自现读文件；叫停文件的路径只有
    /// `stop_file_of_the_environment` 一处读环境变量，给定状态表的引擎起步时经它读一次路径、记进它自己那一个叫停。扫崩溃放量模块族编进库的那几份，不要卡。
    #[test]
    fn the_run_reads_the_stop_file_only_into_the_stop_request_it_remembers() {
        let lines_naming = |name: &str| -> Vec<String> {
            code_lines_outside_the_tests()
                .into_iter()
                .filter(|line| line.contains(name))
                .map(|line| line.trim().to_string())
                .collect()
        };
        assert_eq!(
            lines_naming(concat!("stop_is_requested_", "by_the_environment")),
            vec![
                concat!(
                    "pub fn stop_is_requested_",
                    "by_the_environment() -> bool \u{7b}"
                )
                .to_string(),
                concat!(
                    "let stop_request = StopRequestRememberedInThisRun::new(&stop_is_requested_",
                    "by_the_environment);"
                )
                .to_string(),
            ],
            "读叫停文件在不在的只有定义它的那一处与流水线这一趟记住叫停的那一处"
        );
        assert_eq!(
            lines_naming(concat!(
                "var_os(\"SINGLEFS_CRASH_AMPLIFICATION_",
                "STOP_FILE\")"
            )),
            vec![concat!(
                "std::env::var_os(\"SINGLEFS_CRASH_AMPLIFICATION_",
                "STOP_FILE\").map(PathBuf::from)"
            )
            .to_string()],
            "读叫停文件路径这个环境变量的只有 stop_file_of_the_environment 那一处"
        );
        assert_eq!(
            lines_naming(concat!("stop_file_of_", "the_environment()")),
            vec![
                concat!(
                    "stop_file_of_",
                    "the_environment().is_some_and(|path| path.exists())"
                )
                .to_string(),
                concat!(
                    "pub(super) fn stop_file_of_",
                    "the_environment() -> Option<PathBuf> \u{7b}"
                )
                .to_string(),
                // 给定状态表的引擎（GivenStatesEngine）是另一趟：起的时候读一次路径、记住它自己那一个叫停，崩溃注入领片之前问的是这一个
                concat!("let stop_file = stop_file_of_", "the_environment();").to_string(),
            ],
            "读叫停文件路径的只有流水线那一侧现读在不在的那一处与给定状态表的引擎起步时记一次的那一处"
        );
    }

    /// 第 ① 段在卡上判、照叫停收尾的那一支打头一句 Out of Memory 那一行
    /// （`RoundsOfTheCards::not_run_line_of_a_stop_after_an_out_of_memory`，`gpu_memory_ledger` 钉它的形态）。扫崩溃放量模块族编进库的那几份，不要卡。
    #[test]
    fn the_gpu_rounds_stopped_on_request_print_the_first_out_of_memory_in_the_not_run_line() {
        let lines = code_lines_outside_the_tests();
        let arm = lines
            .iter()
            .position(|line| line.trim() == "AfterARoundOfTheCards::StoppedOnRequest => \u{7b}")
            .expect("第 ① 段几轮卡的收尾里有照叫停收尾那一支");
        let arm_body = lines[arm..]
            .iter()
            .take_while(|line| line.trim() != "break;")
            .cloned()
            .collect::<Vec<String>>()
            .join("\n");
        assert!(
            arm_body.contains(".not_run_line_of_a_stop_after_an_out_of_memory(")
                && arm_body.contains("PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE")
                && arm_body
                    .lines()
                    .any(|line| line.trim() == "println!(\"{not_run_line}\");"),
            "照叫停收尾那一支要把头一句 Out of Memory 那一行打进标准输出（整行比，`eprintln!` 不算）：{arm_body}"
        );
    }

    /// （正推第 3 点：崩溃放量这一侧的接线五种改回去，两条扫源码用例照绿）：这一趟记住的那一个叫停
    /// 一路交到一轮收尾（A）、起下一张卡（B）、卡线程判块那一圈（C）与整趟收尾（D）；E（照叫停收尾那一支打进标准输出）在上一条。扫崩溃放量模块族编进库的那几份，不要卡。
    #[test]
    fn the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run(
    ) {
        let lines: Vec<String> = code_lines_outside_the_tests()
            .iter()
            .map(|line| line.trim().to_string())
            .collect();
        let count = |expected: &str| {
            lines
                .iter()
                .filter(|line| line.as_str() == expected)
                .count()
        };
        for (expected, what) in [
            (
                concat!("let stop = stop_request.", "stop_of_the_run();"),
                "一轮收尾问这一趟记住的叫停（A）",
            ),
            (
                concat!("&|| work.", "stop_of_the_cards(),"),
                "卡线程判块那一圈问卡上的叫停（C）",
            ),
            (
                concat!(
                    "let stop_is_requested = || stop_",
                    "request.is_requested();"
                ),
                "整趟收尾问的是记住的那一个（D）",
            ),
            (
                concat!("let stop_was_requested = stop_", "is_requested();"),
                "整趟收尾照它判提早收尾（D）",
            ),
        ] {
            assert_eq!(count(expected), 1, "{what}：要有一行 {expected}");
        }
        let start = lines
            .iter()
            .position(|line| line.starts_with("fn start_the_next_card_with_states<"))
            .expect("有起下一张卡的那一个函数");
        let next_card_body: Vec<&String> = lines[start..]
            .iter()
            .take_while(|line| line.as_str() != "\u{7d}")
            .collect();
        assert!(
            next_card_body
                .iter()
                .any(|line| line.as_str() == concat!("work.", "stop_of_the_cards()")),
            "起下一张卡照卡上的叫停问（B）：{next_card_body:?}"
        );
    }

    /// 同卡判器按显存排队（这一趟自己起的卡那一支：几条工作线程同时往同一张卡上建判器、各照整张卡的额度开草稿区，叠起来撞过顶）：
    /// 给定状态表与观察者交回的盘两路在自己起的卡上都只经 `judge_on_a_card_started_for_this_run_queued_by_memory` 建判器（各调一次）；
    /// 那个函数先照一条草稿缓冲定宽度、按宽度算要的显存、经排队排进一张卡，再把那张卡的克隆改成排进的额度、在克隆上建判器，
    /// 建判器传的宽度是那一条草稿缓冲（不是 `usize::MAX`）、卡是改过额度的克隆（不是原来那张）；判完先丢判器、再等设备、最后还占位。
    /// 别处在卡上建判器都不传 `usize::MAX`，轮卡的计数没有了。扫崩溃放量模块族编进库的那几份，不要卡。
    #[test]
    fn judges_on_the_cards_started_for_this_run_queue_by_memory_one_scratch_lane_wide_on_a_card_whose_quota_is_the_admitted_need(
    ) {
        let lines_as_written = code_lines_outside_the_tests();
        let lines: Vec<String> = lines_as_written
            .iter()
            .map(|line| line.trim().to_string())
            .collect();
        let start = lines
            .iter()
            .position(|line| {
                line.starts_with(
                    "pub(super) fn judge_on_a_card_started_for_this_run_queued_by_memory(",
                )
            })
            .expect("有在自己起的卡上判的那一个函数");
        // 函数体到顶格的右括号为止（里面的块缩进过，整行比不会提早收）
        let body_length = lines_as_written[start..]
            .iter()
            .position(|line| line.as_str() == "\u{7d}")
            .expect("函数在顶格的右括号收尾");
        let body: Vec<&String> = lines[start..start + body_length].iter().collect();
        let position_of = |expected: &str| -> usize {
            let found: Vec<usize> = body
                .iter()
                .enumerate()
                .filter(|(_, line)| line.as_str() == expected)
                .map(|(line_index, _)| line_index)
                .collect();
            assert_eq!(found.len(), 1, "要恰好一行 {expected}：{body:#?}");
            found[0]
        };
        let width = position_of(
            "let states_per_dispatch = states_per_scratch_lane(smallest_maximum_storage_buffer_binding_size);",
        );
        let need = position_of(
            "let need_mebibytes = mebibytes_a_judge_needs(packed, states_per_dispatch);",
        );
        let admitted = position_of(".admit_on_the_first_card_with_room(need_mebibytes)");
        let quota = position_of("card_with_the_admitted_quota.memory_quota_mebibytes =");
        let judge_built =
            position_of("let judge = GpuCrashJudge::on_card_compiling_these_kernel_sources(");
        let judge_dropped = position_of("drop(judge);");
        let waited = position_of("let waited = card_with_the_admitted_quota");
        let place_returned = position_of("drop(place);");
        assert!(
            width < need
                && need < admitted
                && admitted < quota
                && quota < judge_built
                && judge_built < judge_dropped
                && judge_dropped < waited
                && waited < place_returned,
            "先定宽度、算显存、排进一张卡、改克隆的额度，再建判器；判完丢判器、等设备、还占位：{body:#?}"
        );
        assert_eq!(
            body[judge_built + 1].as_str(),
            "&card_with_the_admitted_quota,",
            "判器建在改过额度的那份克隆上，不在原来那张卡上（原来那张的额度是整张卡）"
        );
        assert_eq!(
            body[judge_built + 3].as_str(),
            "states_per_dispatch,",
            "建判器传的宽度是一条草稿缓冲那么宽，不是 usize::MAX"
        );
        let callers = lines
            .iter()
            .filter(|line| {
                line.contains("judge_on_a_card_started_for_this_run_queued_by_memory(")
                    && !line.starts_with("pub(super) fn ")
            })
            .count();
        assert_eq!(
            callers, 2,
            "给定状态表与观察者交回的盘两路各调一次在自己起的卡上判的那一个函数"
        );
        for (line_index, line) in lines.iter().enumerate() {
            if line.contains("GpuCrashJudge::on_card_") {
                // 一次调用到它收尾的那一行为止（调用写在一行里的就是那一行）
                let mut call: Vec<&String> = Vec::new();
                for call_line in &lines[line_index..] {
                    call.push(call_line);
                    if call_line.ends_with(");") || call_line.starts_with(')') {
                        break;
                    }
                }
                assert!(
                    call.iter()
                        .all(|call_line| !call_line.contains("usize::MAX")),
                    "在卡上建判器不传 usize::MAX（那是照整张卡的额度开草稿区）：{call:#?}"
                );
            }
        }
        assert!(
            lines
                .iter()
                .all(|line| !line.contains(".next_card") && !line.contains("next_card %")),
            "自己起的卡不再按计数轮着给，改由排队给"
        );
    }

    /// （正推六·二：交给第 ① 段在卡上判与 CPU 判器的叫停那一行换成永不叫停，
    /// 上一条照绿）：这一趟记住的那一个叫停（`stop_is_requested`）就是交给 `judge_my_blocks_on_gpu` 与 `check_selected_blocks` 的那一个
    /// （两处调用的最后一个参数整行比）。下游各层把收到的叫停原样往下交，换掉就是那一层的参数没用上、编译器报没用的变量。扫崩溃放量模块族编进库的那几份，不要卡。
    #[test]
    fn the_stop_request_remembered_in_this_run_is_the_one_handed_to_the_gpu_judge_and_the_cpu_checker(
    ) {
        let lines: Vec<String> = code_lines_outside_the_tests()
            .iter()
            .map(|line| line.trim().to_string())
            .collect();
        for (call_opening, what) in [
            (
                concat!("let gpu_judging = judge_my_blocks_on_gpu", "("),
                "第 ① 段在卡上判",
            ),
            (
                concat!("let checking = check_selected_blocks", "("),
                "第 ① 段 CPU 判器",
            ),
        ] {
            let openings: Vec<usize> = lines
                .iter()
                .enumerate()
                .filter(|(_line_index, line)| line.as_str() == call_opening)
                .map(|(line_index, _line)| line_index)
                .collect();
            assert_eq!(openings.len(), 1, "{what}：族里有一处 {call_opening}");
            let arguments: Vec<&String> = lines[openings[0] + 1..]
                .iter()
                .take_while(|line| !line.starts_with(')'))
                .collect();
            assert_eq!(
                arguments.last().map(|argument| argument.as_str()),
                Some(concat!("&stop_is_", "requested,")),
                "{what}收到的叫停是这一趟记住的那一个：{arguments:?}"
            );
        }
    }

    /// （正推：下游各层把收到的叫停换成永不叫停，`cargo check` 只报没用的变量、
    /// 变异跑道不带 `-D warnings`，照绿）：崩溃放量模块族编进库的那几份里收叫停参数（`stop_is_requested: &(dyn Fn() -> bool + Sync),`，测试那一段之外）的函数
    /// 照文件表的次序（[`CRASH_AMPLIFICATION_FAMILY_FILES`]，路径的字节序）一份接一份、每份里照源码次序列成名单（多一层少一层都要回来看罩没罩到）。
    /// （「参数行之后整名用上至少一次」被只提一次名字而永不叫停的写法骗过，
    /// 正推 A7 `let _ = stop_is_requested`、攻方 crash-2 `&|| { let _ = stop_is_requested; false }`、crash-3 `if false && stop_is_requested()`）：
    /// 不要卡的两层改由行为用例数叫停被问了几次——`check_pending_blocks_in_pieces` 由本模块
    /// `the_stop_request_is_asked_once_before_each_block_and_a_stop_leaves_the_later_blocks_untaken`，`check_blocks_where_until` 由
    /// `tests/crash_amplification_pipeline_records_checks_and_reuses.rs` 的 `a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest`
    /// （叫停在第二次被问时交真，只判了一块）。另四层换不成不要卡的行为用例：第 ① 段在卡上判那三层要卡，`check_selected_blocks` 要录好的流与库
    /// （只有 `run_from_environment` 调它，经环境跑的那几条用例里叫停文件放上时块都判过了，问不到这一层），这里扫源码：参数行之后到函数收尾的
    /// 顶格右括号，用到叫停整名的行（去掉首尾空白）逐行等于钉住的那几行，往下交的那一行之后紧跟右括号（是调下一层的最后一个实参）。
    /// 整行比拦得住上面三种写法（多一行、或那一行不是原样）。
    /// （正推 N1：`check_selected_blocks` 里真调用的叫停换成 `&|| false`，钉住的那一行
    /// 连同它后面的右括号原样留在 `if false { … }` 死分支里的另一次调用里，编得过、整行比照绿）：函数体短的三层
    /// （`judge_my_blocks_on_gpu_cards`、`check_selected_blocks`、`judge_my_blocks_on_gpu`）改成整句比——参数行之后到函数收尾
    /// （返回类型与函数体，去掉注释行与空白）整个等于钉住的那一句，死分支、多一次调用、换实参、换被调的函数都红；函数体太长、整句比不了的
    /// `judge_my_blocks_on_the_cards_started_for_this_run` 照旧整行比那一行，另要它是函数体顶层的一句
    /// （不在任何块里：`if false { … }`、`if false && … { … }` 包着的红）、`stop_request` 在这一层只由它 `let` 绑一次
    /// （后面再 `let` 一次就换掉了记住的叫停）、`StopRequestRememberedInThisRun` 在这一层只出现在它里面（块里 `use` 或另起同名类型会遮住它）。
    /// （第十六至十九块代码轮判决 K4，正推 K4-3：名单外三种写法拦不住）：
    /// 收叫停的层改成按叫停类型找（`&(dyn Fn() -> bool + Sync)`，空白随意、可以跨行），名单记层名连同参数名——参数与别的参数挤在签名那一行
    /// （`the_stop_request_scan_finds_a_new_layer_whose_stop_parameter_shares_the_signature_line`）、参数换了名字
    /// （`the_stop_request_scan_finds_a_new_layer_whose_stop_parameter_is_renamed`）的新层都在名单外、红；没编进 `gpu` 特性时那一份
    /// `judge_my_blocks_on_gpu`（参数 `_stop_is_requested`）因此也进了名单，整句钉成只交错；长的那一层数 `stop_request` 被 `let` 绑了几次
    /// 改成看每个 `let`（`if let`、`while let` 也算）到 `=` 或 `;` 之前那一截里有没有整个名字 `stop_request`——解构
    /// `let (stop_request, …) = …`、`let` 后面多几个空格都数得到
    /// （`the_stop_request_scan_finds_the_remembered_stop_request_rebound_by_a_tuple_pattern_or_a_let_with_extra_spaces`）；
    /// 数花括号的层数认得原始字符串与块注释（`brace_depth_reads_past_raw_strings_block_comments_and_strings_spanning_lines`），
    /// 长的那一层里写原始字符串不再让这条用例当场红，`if false { /* } */` 也骗不过层数。
    /// 测试模块只从代码里那一处头两行切（块注释里写一份头两行、藏住它与真测试模块
    /// 之间的新层，攻方 K4-2 v2，`the_tests_module_is_cut_at_its_opening_in_code_not_at_a_copy_in_a_comment_or_a_string`）；
    /// 找层、数 `let`、数用到叫停的行都在注释与字面量换成空格之后的代码上做，整句、整行比在去掉注释的代码上做——签名行尾注释里的 `fn`
    /// 不再被当成层名，行尾注释、字符串里的 `let stop_request` 与钉住那一行、短层函数体上的行尾注释不再判红
    /// （`the_stop_request_scan_reads_layer_names_rebindings_and_pinned_lines_past_comments_and_strings`）；钉住那一行与叫停参数上挂属性
    /// （`#[cfg(any())]` 编译时去掉那一句、那个参数，正推 K4-1 A01）判红（`the_stop_request_scan_finds_an_attribute_on_the_pinned_line_or_on_a_stop_parameter`）；
    /// 叫停写成钉住之外的类型（多加 `Send`、`'static`，`&mut`、`&'static`，类型里夹注释，`Box<dyn …>`，`dyn Sync + Fn() -> bool`，
    /// `&dyn Fn() -> bool`，`impl`、泛型、族里这几份源文件里的类型别名，函数指针）的新层照 `Fn() -> bool` 这个形找到、判红
    /// （`the_stop_request_scan_finds_new_layers_whose_stop_type_is_written_another_way`，正推 K4-2 C02–C12）。
    /// （K4）：测试模块的头两行只认按行逐字、括号第 0 层、前一个词是条目收尾的那一处，
    /// 拿一个吃掉 `#[cfg(test)]` 的宏把测试模块包进宏调用的圆括号里（正推 K4-1 S14）、包着的假测试模块里藏一层（S19）都红，
    /// 恰好一处的上界与第 0 层两道各有用例钉（正推 K3-5 W8、W9、X6、X7，`the_tests_module_is_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item`）；
    /// 叫停闭包的返回类型写成 `::core::primitive::bool`、`std::primitive::bool`、套括号的 `(bool)` 的新层照形找到、红
    /// （正推 K4-1 N01、N02，`the_stop_request_scan_finds_new_layers_whose_stop_returns_bool_by_a_path_or_in_parentheses`）。
    /// 批 16 G4 抽出 CPU 判段共用一份枚举表的那一层（`check_pending_blocks_sharing_one_enumeration_table_of_the_flow`）：
    /// 它只把收到的叫停原样交给 `check_pending_blocks_in_pieces`，`check_blocks_where_until` → `check_blocks_where_until_with_threads` → 它，
    /// 在 `a_run_asked_to_stop_keeps_the_block_it_judged_and_the_next_run_judges_only_the_rest` 那条行为用例的路上，交下去的换成永不叫停由那条用例判红。
    /// 观察者录入段是两层：`record_the_observers_of_the_flow` 取核对线程数、把收到的叫停原样交给 `record_the_observers_of_the_flow_with_threads`，
    /// 函数体短，照短的几层整句比；录入的循环与每录一块之前问一次叫停在后一层，只进名单。
    /// 复用哨兵抽中强制现判、录入块不在库里的那一块回录入段重录是一层（`record_the_observers_of_the_forced_block_again`）：
    /// 它把收到的叫停原样交给 `record_the_observers_of_the_flow`，没有数叫停被问了几次的行为用例，函数体短，照短的几层整句比。
    /// CPU 判段判一段交调用方给的判器是一层（`check_blocks_where_until_judging_each_piece_with`，库每次读写现取现放）：
    /// 它把收到的叫停原样交给 `check_pending_blocks_sharing_one_enumeration_table_of_the_flow`，同在上面那条行为用例的路上。
    /// 判段补录照判不入库的观察者是一层（`record_the_observers_kept_out_of_the_library_again`），观察者与哨兵的判段是一层
    /// （`judge_the_observers_then_the_reuse_sentinels`，原样交给补录那一层）：这两层只进名单，交下去的换成永不叫停
    /// 由行为用例 `a_stop_while_recording_again_the_observers_kept_out_of_the_library_leaves_them_unjudged` 判红。
    /// 配了 GPU 的判段写待判行、等服务判完是一层（`judge_my_blocks_through_the_pending_rows`，每现算一遍判完问一次叫停）：只进名单，
    /// 不问叫停由行为用例 `a_stop_while_waiting_for_the_gpu_verification_server_leaves_the_pending_rows_and_returns` 判红；
    /// 它的入口 `judge_my_blocks_on_gpu`（两份，没编进 gpu 特性那一份只交错）函数体短，照短的几层整句比。
    /// 配了 GPU 的第 ② 段写核对待判行、等服务核完是一层（`verify_my_blocks_through_the_facts_verifier_pending_rows`，每现算一遍核完问一次叫停），
    /// 只进名单：不问叫停由行为用例 `a_stop_while_waiting_for_the_facts_verifier_leaves_the_facts_verifier_pending_rows_and_returns` 判红；
    /// 它的入口 `verify_my_blocks_on_gpu`（两份，没编进 gpu 特性那一份只交错）函数体短，照短的几层整句比。
    /// 没交卡的判段（给定状态表与观察者交回的盘）写判表待判行、等服务写回判表结论字是一层
    /// （`wait_for_the_verdict_words_of_the_packed_tables`，每现算一遍问一次叫停），登记临时库、包它的那一层
    /// （`judge_packed_tables_through_the_gpu_verification_server`，原样交下去）：两层只进名单。两种调用方都交永不叫停
    /// （`WayToTheGpuVerificationServer::judge_the_packed_tables`），这两层不问叫停、不原样交下去的写法没有行为用例判红。名单因此是二十层。
    ///
    /// 拦不住的写法，一种一条：
    /// - 同一行原样交下去而下一层的函数体里不用（下一层由它自己那一条管；第 ① 段在卡上判那一层里 `stop_request` 往下怎么用由
    ///   `the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run` 管，也只看文本）；
    /// - 长的那一层：钉住那一行之前提早返回，或让它之后的路走不到；`stop_request` 经 `let` 之外的绑定（`match` 的臂、闭包参数、`for`）
    ///   在块里换成别的；
    /// - 被调的那一层自己改了（短的几层整句比钉的是调谁、怎么交，不是被调的函数里做什么）；`gpu_memory_ledger` 的 `StopRequestRememberedInThisRun::new` 改了；
    /// - 叫停的类型写成别的源文件、别的 crate 里定义的别名或 trait（`&crate::某处::叫停类型`），或换成不带 `Fn() -> bool` 字样的类型
    ///   （`&StopRequestRememberedInThisRun`、自己的 trait）的新层：只扫族里这几份源文件、只认这个形；
    /// - 叫停闭包的返回类型写成别名（`type 别名 = bool;` 之后 `Fn() -> 别名`，正推 K4-1 N03）或类型宏（`Fn() -> 某宏!()`，N15）的新层：
    ///   只认 `bool`、按路径写到 `core`、`std` 里的那个原始类型与外面套的括号，别名与宏展开出来的是什么，扫族里这几份源文件看不出；
    /// - 在函数上面加 `#[cfg]`，让照上一条另写类型的同名函数编进来（同样写成钉住的类型或族里这几份源文件里看得见的形的，读到的名单多一层、红）；
    /// - 长的那一层里宏展开出来的调用与绑定；
    /// - 过程宏（属性宏）挂在测试模块上、或把它包住，把测试模块编进非测试构建：切口只认头两行的写法与所在的层，宏做了什么扫源码看不见。
    ///
    /// 合法写法判红、是故意拒的，一种一条：
    /// - 测试模块头两行换写法（`mod tests` 后面不空格就接左花括号、写在一行、`#[ cfg ( test ) ]`、多空格、缩进、行尾带空白或注释）
    ///   与 CRLF 换行（[`text_outside_the_tests_module`]，两份文件是 rustfmt 出的，头两行就是一种写法）；
    /// - 叫停闭包参数之外、不带参数而返回以 `bool` 开头的元组的闭包（`Fn -> (bool, u8)`写明第二十四、
    ///   二十五块，正推 `N16`）：[`end_of_a_bool_return_type`] 跳过开头的左括号读到 `bool` 就算，不往后找与它配对的
    ///   右括号，分不出 `(bool)` 与以 `bool` 开头的元组；要分就得把返回类型照类型的语法读到收尾（元组里还能套泛型、路径、括号），这一扫
    ///   只认形，方向是不过就红、不会放过一层。族里这几份没有这样的闭包，要写就把返回类型写成具名的结构体。
    #[test]
    fn every_layer_handed_the_stop_request_hands_it_on_or_asks_it() {
        assert_eq!(
            ways_a_layer_handed_the_stop_request_could_drop_it(&library_files_of_the_family()),
            Vec::<String>::new(),
            "收叫停的每一层照钉住的那样往下交或问"
        );
    }

    /// （第十六至十九块代码轮判决 K4，正推 K4-3 第 1 种 `bypass_extra_layer_one_line`）：新加一层收叫停、
    /// 叫停参数与别的参数挤在签名那一行里，照样找得到这一层，它不在名单里就红。扫崩溃放量模块族编进库的那几份（其中一份改过），不要卡。
    #[test]
    fn the_stop_request_scan_finds_a_new_layer_whose_stop_parameter_shares_the_signature_line() {
        let new_layer = concat!(
            "fn an_extra_layer_never_asking(first_input: u8, stop_is_",
            "requested: &(dyn Fn() -> bool + Sync)) -> bool \u{7b} first_input == 0 \u{7d}"
        );
        let root_with_the_new_layer = format!("{}\n{new_layer}\n", source_of_the_family_root());
        let found = ways_a_layer_handed_the_stop_request_could_drop_it(
            &library_files_with_one_replaced(ROOT_OF_THE_FAMILY, &root_with_the_new_layer),
        );
        assert!(
            found
                .iter()
                .any(|finding| finding.contains("(\"an_extra_layer_never_asking\", ")),
            "签名一行里收叫停的新层进了读到的名单、与钉住的名单对不上：{found:#?}"
        );
    }

    /// （第十六至十九块代码轮判决 K4，正推 K4-3 第 2 种 `bypass_extra_layer_other_name`）：新加一层收叫停、
    /// 叫停参数换了名字，照样找得到这一层，它不在名单里就红。扫崩溃放量模块族编进库的那几份（其中一份改过），不要卡。
    #[test]
    fn the_stop_request_scan_finds_a_new_layer_whose_stop_parameter_is_renamed() {
        let new_layer = concat!(
            "fn an_extra_layer_with_a_renamed_stop(\n",
            "    first_input: u8,\n",
            "    stop: &(dyn Fn() -> bool + Sync),\n",
            ") -> bool \u{7b}\n",
            "    first_input == 0\n",
            "\u{7d}"
        );
        let root_with_the_new_layer = format!("{}\n{new_layer}\n", source_of_the_family_root());
        let found = ways_a_layer_handed_the_stop_request_could_drop_it(
            &library_files_with_one_replaced(ROOT_OF_THE_FAMILY, &root_with_the_new_layer),
        );
        assert!(
            found
                .iter()
                .any(|finding| finding
                    .contains("(\"an_extra_layer_with_a_renamed_stop\", \"stop\")")),
            "参数换了名字的新层进了读到的名单、与钉住的名单对不上：{found:#?}"
        );
    }

    /// （第十六至十九块代码轮判决 K4，正推 K4-3 第 3 种 `bypass_tuple_let_rebind`、`bypass_let_extra_spaces`）：
    /// 长的那一层钉住那一行之后解构重绑 `stop_request`、或 `let` 与名字之间多几个空格重绑，都数成又 `let` 了一次。扫崩溃放量模块族编进库的那几份（其中一份改过），不要卡。
    #[test]
    fn the_stop_request_scan_finds_the_remembered_stop_request_rebound_by_a_tuple_pattern_or_a_let_with_extra_spaces(
    ) {
        let pinned_line = concat!(
            "    let stop_request = StopRequestRememberedInThisRun::new(stop_is_",
            "requested);\n"
        );
        assert_eq!(
            occurrences_in_the_library_files(pinned_line),
            1,
            "长的那一层钉住的那一行在崩溃放量模块族编进库的那几份里恰好一处"
        );
        let (path, source) =
            first_library_file_holding(pinned_line).expect("钉住的那一行在族里编进库的一份里");
        for (form, rebinding) in [
            (
                "解构",
                "    let (stop_request, _unused) = (the_never_stopping_request(), 0);\n",
            ),
            (
                "多几个空格",
                "    let   stop_request  =  the_never_stopping_request();\n",
            ),
        ] {
            let rebound = source.replacen(pinned_line, &format!("{pinned_line}{rebinding}"), 1);
            let found = ways_a_layer_handed_the_stop_request_could_drop_it(
                &library_files_with_one_replaced(path, &rebound),
            );
            assert!(
                found.iter().any(|finding| finding.contains(
                    "里 stop_request 只由钉住的那一行 let 绑一次（后面再 let 一次、解构、if let 就换掉了记住的叫停），读到 2 次"
                )),
                "{form}重绑数成第二次 let：{found:#?}"
            );
        }
    }

    /// （第十六至十九块代码轮正推 K5：数花括号遇到原始字符串当场 `assert!`）：原始字符串（里面的 `"`、
    /// 收尾的反斜杠、跨行）、块注释（嵌套、里面的右花括号）、跨行的普通字符串里的花括号都不算层数。不要卡。
    #[test]
    fn brace_depth_reads_past_raw_strings_block_comments_and_strings_spanning_lines() {
        let lines = [
            "let path = r\"C:\\\";",
            "if path.is_empty() \u{7b}",
            "    let quoted = r#\"say \"\u{7b}\" here\"#;",
            "    let bytes = br##\"\u{7d}\"#\u{7d}\"##;",
            "    let spanning = r#\"",
            "\u{7b}\u{7b}",
            "\"#;",
            "    /* a \u{7d} in a comment */",
            "    let nested = /* outer /* inner \u{7d} */ still \u{7d} */ 0;",
            "    let plain = \"",
            "\u{7b}\";",
            "\u{7d}",
            "after",
        ];
        assert_eq!(
            brace_depth_at_the_start_of_each_line(&lines),
            vec![0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
            "只有 if 那一行的左花括号与倒数第二行的右花括号算层数：{lines:#?}"
        );
    }

    /// （v2）：真测试模块头两行之前写一份在块注释里（或跨行的字符串里），
    /// 中间放一层收叫停而不问的新层、一个按名字读名单外环境变量的函数：测试模块只从代码里那一处切，两样都照样扫得到。扫崩溃放量模块族编进库的那几份（其中一份改过），不要卡。
    #[test]
    fn the_tests_module_is_cut_at_its_opening_in_code_not_at_a_copy_in_a_comment_or_a_string() {
        let source = source_of_the_family_root();
        assert_eq!(
            source.matches(TESTS_MODULE_OPENING).count(),
            1,
            "族的根里测试模块的头两行原文只有代码里那一处"
        );
        let new_layer = concat!(
            "pub fn an_extra_layer_never_asking(first_input: u8, stop_is_",
            "requested: &(dyn Fn() -> bool + Sync)) -> bool \u{7b} let _ = stop_is_",
            "requested; first_input == 0 \u{7d}\n"
        );
        let reading_a_variable_left_out = concat!(
            "pub fn reads_a_variable_left_out_of_the_list() -> bool \u{7b} ",
            "std::env::var(\"SINGLEFS_A_VARIABLE_LEFT_OUT_OF_THE_LIST\").is_ok() \u{7d}\n"
        );
        for (form, copy_of_the_opening) in [
            ("块注释", format!("/*\n{TESTS_MODULE_OPENING}\n*/\n")),
            (
                "跨行的字符串",
                format!(
                    "pub const A_COPY_OF_THE_OPENING: &str = \"\n{TESTS_MODULE_OPENING}\n\";\n"
                ),
            ),
        ] {
            let hiding = source.replacen(
                TESTS_MODULE_OPENING,
                &format!(
                    "{copy_of_the_opening}{new_layer}{reading_a_variable_left_out}{TESTS_MODULE_OPENING}"
                ),
                1,
            );
            let family_hiding = library_files_with_one_replaced(ROOT_OF_THE_FAMILY, &hiding);
            let found = ways_a_layer_handed_the_stop_request_could_drop_it(&family_hiding);
            assert!(
                found
                    .iter()
                    .any(|finding| finding.contains("(\"an_extra_layer_never_asking\", ")),
                "{form}里写一份测试模块的头两行，藏不住它与真测试模块之间收叫停的新层：{found:#?}"
            );
            let names_read = environment_variables_read_by_name_in(
                &family_text_outside_the_tests(&family_hiding).join("\n"),
            );
            assert!(
                names_read
                    .iter()
                    .any(|name| name == "SINGLEFS_A_VARIABLE_LEFT_OUT_OF_THE_LIST"),
                "{form}里写一份测试模块的头两行，藏不住它与真测试模块之间按名字读的环境变量：{names_read:?}"
            );
        }
    }

    /// 跑 `check`，它 panic 时交 panic 的消息，不 panic 交 `None`：扫源码的用例看判法在改过的源码上红没红、红在哪一句。
    fn panic_message_of(check: impl FnOnce() + std::panic::UnwindSafe) -> Option<String> {
        std::panic::catch_unwind(check).err().map(|payload| {
            payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| {
                    payload
                        .downcast_ref::<&str>()
                        .map(|message| (*message).to_string())
                })
                .unwrap_or_default()
        })
    }

    /// （K4，正推 K3-5 `X6`、`X7`，K4-1 `S14`、`S19`）：真测试模块头两行之前
    /// 写一份头两行，认不认照 [`text_outside_the_tests_module`] 那几条。宏调用圆括号里的（前一个词是左括号；里面藏一层收叫停的层，`S19`，
    /// 那一层留在测试模块之外、照样找得到；前面先放一条以分号收尾的条目）、花括号里的（前面先放一条以分号收尾的条目）、前一个词不是条目收尾的，
    /// 都不认，切口照旧在真测试模块；代码里两处都认的（恰好一处的上界）、真测试模块包进宏调用的圆括号（`S14`）、头两行换写法与 CRLF 换行
    /// （按行逐字的口径，故意拒），都红在「恰好一处」那一句。扫崩溃放量模块族编进库的那几份（其中一份改过），不要卡。
    /// （第二节 K2，攻方 B2：括号那一道只数圆括号照绿）：真测试模块包进方括号的
    /// 宏调用（`keep_hidden![` 起、前面先放一条以分号收尾的条目、`];` 收尾，攻方 `c09`；rustc 编得过、非测试构建里照编），也红在「恰好一处」。
    #[test]
    fn the_tests_module_is_cut_only_at_one_opening_of_two_whole_lines_at_the_top_level_after_an_item(
    ) {
        let source = source_of_the_family_root();
        let outside_the_tests = text_outside_the_tests_module(source);
        let hidden_layer = concat!(
            "    pub fn a_layer_hidden_in_a_wrapped_copy_of_the_tests_module(stop: &(dyn Fn() -> bool + Sync)) -> bool ",
            "\u{7b} stop() \u{7d}\n"
        );
        let wrapped_copy_hiding_a_layer =
            format!("keep_hidden!(\n{TESTS_MODULE_OPENING}\n{hidden_layer}") + "\u{7d}\n);\n";
        for (form, copy_of_the_opening) in [
            (
                "宏调用圆括号里的一份，里面藏一层收叫停的层（S19）",
                wrapped_copy_hiding_a_layer.as_str(),
            ),
            (
                "宏调用圆括号里、前面先放一条以分号收尾的条目的一份",
                "keep_hidden!(\nconst BEFORE_THE_COPY: u8 = 0;\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n);\n",
            ),
            (
                "花括号里、前面先放一条以分号收尾的条目的一份",
                "mod a_module_holding_a_copy \u{7b}\nconst BEFORE_THE_COPY: u8 = 0;\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n\u{7d}\n",
            ),
            (
                "前一个词不是条目收尾的一份",
                "static NOT_ENDED_BY_A_SEMICOLON: u8 = 0\n#[cfg(test)]\nmod tests \u{7b}\n\u{7d}\n",
            ),
        ] {
            let with_a_copy = source.replacen(
                TESTS_MODULE_OPENING,
                &format!("{copy_of_the_opening}{TESTS_MODULE_OPENING}"),
                1,
            );
            assert_eq!(
                text_outside_the_tests_module(&with_a_copy).len(),
                outside_the_tests.len() + copy_of_the_opening.len(),
                "{form}不认，切口照旧在真测试模块"
            );
        }
        let with_a_wrapped_copy_hiding_a_layer = source.replacen(
            TESTS_MODULE_OPENING,
            &format!("{wrapped_copy_hiding_a_layer}{TESTS_MODULE_OPENING}"),
            1,
        );
        let found =
            ways_a_layer_handed_the_stop_request_could_drop_it(&library_files_with_one_replaced(
                ROOT_OF_THE_FAMILY,
                &with_a_wrapped_copy_hiding_a_layer,
            ));
        assert!(
            found
                .iter()
                .any(|finding| finding
                    .contains("a_layer_hidden_in_a_wrapped_copy_of_the_tests_module")),
            "宏包着的假测试模块里藏的那一层找得到、红（S19）：{found:#?}"
        );
        let opening_offset = source
            .find(TESTS_MODULE_OPENING)
            .expect("族的根里有测试模块的头两行");
        let end_of_the_tests_module = opening_offset
            + source[opening_offset..]
                .find(TOP_LEVEL_CLOSING_BRACE_LINE)
                .expect("测试模块在顶格的右括号行收尾")
            + TOP_LEVEL_CLOSING_BRACE_LINE.len();
        let tests_wrapped_in_square_brackets_after_an_item =
            String::from(&source[..opening_offset])
                + "keep_hidden![\nconst BEFORE_THE_TESTS: u8 = 0;\n"
                + &source[opening_offset..end_of_the_tests_module]
                + "];\n"
                + &source[end_of_the_tests_module..];
        let mut refused_sources: Vec<(&str, String)> = vec![
            (
                "代码里两处都认的头两行",
                source.replacen(
                    TESTS_MODULE_OPENING,
                    &format!(
                        "{TESTS_MODULE_OPENING}{TOP_LEVEL_CLOSING_BRACE_LINE}{TESTS_MODULE_OPENING}"
                    ),
                    1,
                ),
            ),
            (
                "真测试模块包进宏调用的圆括号（S14）",
                source.replacen(
                    TESTS_MODULE_OPENING,
                    &format!("keep_hidden!(\n{TESTS_MODULE_OPENING}"),
                    1,
                ),
            ),
            (
                "真测试模块包进宏调用的圆括号、前面先放一条以分号收尾的条目",
                source.replacen(
                    TESTS_MODULE_OPENING,
                    &(String::from("keep_hidden!(\nconst BEFORE_THE_TESTS: u8 = 0;\n")
                        + TESTS_MODULE_OPENING),
                    1,
                ),
            ),
            (
                "真测试模块包进宏调用的花括号、前面先放一条以分号收尾的条目",
                source.replacen(
                    TESTS_MODULE_OPENING,
                    &(String::from("keep_hidden! \u{7b}\nconst BEFORE_THE_TESTS: u8 = 0;\n")
                        + TESTS_MODULE_OPENING),
                    1,
                ),
            ),
            (
                "真测试模块包进宏调用的方括号、前面先放一条以分号收尾的条目（攻方 c09）",
                tests_wrapped_in_square_brackets_after_an_item,
            ),
            ("CRLF 换行", source.replace('\n', "\r\n")),
        ];
        for (form, other_writing) in [
            ("mod 行少一个空格", "#[cfg(test)]\nmod tests\u{7b}"),
            ("写在一行", "#[cfg(test)] mod tests \u{7b}"),
            ("属性里多空格", "#[ cfg ( test ) ]\nmod tests \u{7b}"),
            ("mod 行多空格", "#[cfg(test)]\nmod  tests  \u{7b}"),
            ("头一行缩进", "    #[cfg(test)]\nmod tests \u{7b}"),
            ("头一行行尾带空白", "#[cfg(test)]  \nmod tests \u{7b}"),
            ("头一行行尾带注释", "#[cfg(test)] // 单测\nmod tests \u{7b}"),
            ("第二行行尾带注释", "#[cfg(test)]\nmod tests \u{7b} // 单测"),
        ] {
            refused_sources.push((
                form,
                source.replacen(TESTS_MODULE_OPENING, other_writing, 1),
            ));
        }
        for (form, refused_source) in &refused_sources {
            let message = panic_message_of(|| {
                drop(text_outside_the_tests_module(refused_source));
            });
            assert!(
                message
                    .as_deref()
                    .is_some_and(|message| message.contains("测试模块的头两行恰好一处")),
                "{form}：切口红在「恰好一处」那一句，读到 {message:?}"
            );
        }
    }

    /// （N01、N02：叫停闭包的返回类型写成 `::core::primitive::bool`、
    /// `(bool)` 的新层，叫停原样可用，按形找不到、判绿）：返回类型按路径写到 `core`、`std` 里那个原始类型的、外面套括号的，
    /// 照 [`end_of_a_bool_return_type`] 认成 `bool`，新层找得到、红。扫崩溃放量模块族编进库的那几份（其中一份改过），不要卡。
    #[test]
    fn the_stop_request_scan_finds_new_layers_whose_stop_returns_bool_by_a_path_or_in_parentheses()
    {
        let source = source_of_the_family_root();
        for (marker, new_layer) in [
            (
                "an_extra_layer_whose_stop_returns_bool_by_the_core_path",
                "fn an_extra_layer_whose_stop_returns_bool_by_the_core_path(stop: &(dyn Fn() -> ::core::primitive::bool + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_returns_bool_by_a_relative_core_path",
                "fn an_extra_layer_whose_stop_returns_bool_by_a_relative_core_path(stop: &(dyn Fn() -> core::primitive::bool + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_returns_bool_by_the_std_path",
                "fn an_extra_layer_whose_stop_returns_bool_by_the_std_path(stop: &(dyn Fn() -> std :: primitive :: bool + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_returns_bool_in_parentheses",
                "fn an_extra_layer_whose_stop_returns_bool_in_parentheses(stop: &(dyn Fn() -> (bool) + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_returns_bool_by_a_path_in_two_parentheses",
                "fn an_extra_layer_whose_stop_returns_bool_by_a_path_in_two_parentheses(stop: &(dyn Fn() -> ( ( ::std::primitive::bool ) ) + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_a_function_pointer_returning_bool_in_parentheses",
                "fn an_extra_layer_whose_stop_is_a_function_pointer_returning_bool_in_parentheses(stop: fn() -> (bool)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_fn_mut_returning_bool_by_a_path",
                "fn an_extra_layer_whose_stop_is_fn_mut_returning_bool_by_a_path(stop: &mut dyn FnMut() -> core::primitive::bool) -> bool \u{7b} false \u{7d}",
            ),
        ] {
            let source_with_the_new_layer = format!("{source}\n{new_layer}\n");
            let found = ways_a_layer_handed_the_stop_request_could_drop_it(
                &library_files_with_one_replaced(ROOT_OF_THE_FAMILY, &source_with_the_new_layer),
            );
            assert!(
                found.iter().any(|finding| finding.contains(marker)),
                "叫停闭包的返回类型换了写法的新层 {marker} 找得到、红：{found:#?}"
            );
        }
    }

    /// （A01：长的那一层钉住那一行上面挂 `#[cfg(any)]`，模块里另放一个
    /// 叫 `stop_request` 的静态量，编译时那一行去掉、下面用的是静态量，整行比、层数、`let` 次数照绿）：钉住那一行前面紧挨着的（跳过注释）
    /// 要是上一句的收尾，挂着属性（一行的、跨行的、属性与那一行之间夹注释行的）都红；叫停参数前面挂属性同样红。扫崩溃放量模块族编进库的那几份（其中一份改过），不要卡。
    #[test]
    fn the_stop_request_scan_finds_an_attribute_on_the_pinned_line_or_on_a_stop_parameter() {
        let pinned_line = concat!(
            "    let stop_request = StopRequestRememberedInThisRun::new(stop_is_",
            "requested);\n"
        );
        assert_eq!(
            occurrences_in_the_library_files(pinned_line),
            1,
            "长的那一层钉住的那一行在崩溃放量模块族编进库的那几份里恰好一处"
        );
        let (path, source) =
            first_library_file_holding(pinned_line).expect("钉住的那一行在族里编进库的一份里");
        for (form, attribute) in [
            ("一行的属性", "    #[cfg(any())]\n"),
            ("跨行的属性", "    #[cfg(\n        any()\n    )]\n"),
            (
                "属性与那一行之间夹注释行",
                "    #[cfg(any())]\n    // 编译时这一句去掉\n",
            ),
        ] {
            let under_an_attribute =
                source.replacen(pinned_line, &format!("{attribute}{pinned_line}"), 1);
            let found = ways_a_layer_handed_the_stop_request_could_drop_it(
                &library_files_with_one_replaced(path, &under_an_attribute),
            );
            assert!(
                found
                    .iter()
                    .any(|finding| finding.contains("前面紧挨着的是上一句的收尾")),
                "钉住那一行上面挂{form}，红：{found:#?}"
            );
        }
        let stop_parameter = concat!("\n    stop_is_", "requested: &(dyn Fn() -> bool + Sync),\n");
        // 照文件表的次序头一份有单独占一行的叫停参数的，在它头一处叫停参数上挂属性
        let (parameter_path, parameter_source) = first_library_file_holding(stop_parameter)
            .expect("族里编进库的一份里有单独占一行的叫停参数");
        let parameter_under_an_attribute = parameter_source.replacen(
            stop_parameter,
            &format!("\n    #[cfg(any())]{stop_parameter}"),
            1,
        );
        assert_ne!(
            parameter_under_an_attribute, parameter_source,
            "造的源码在叫停参数上挂了属性"
        );
        let found = ways_a_layer_handed_the_stop_request_could_drop_it(
            &library_files_with_one_replaced(parameter_path, &parameter_under_an_attribute),
        );
        assert!(
            found.iter().any(|finding| finding
                .contains(concat!("的叫停参数 stop_is_", "requested 前面挂着属性"))),
            "叫停参数上面挂属性，红：{found:#?}"
        );
    }

    /// （C02–C12）：新加一层收叫停、叫停写成钉住的
    /// `&(dyn Fn() -> bool + Sync)` 之外的类型，照 `Fn() -> bool` 这个形（`FnMut`、`FnOnce`、函数指针也算）找到、红；类型里夹注释的，
    /// 注释不算，照钉住的类型找到这一层、它不在名单里红。扫崩溃放量模块族编进库的那几份（其中一份改过），不要卡。
    #[test]
    fn the_stop_request_scan_finds_new_layers_whose_stop_type_is_written_another_way() {
        let source = source_of_the_family_root();
        for (marker, new_layer) in [
            (
                "an_extra_layer_whose_stop_is_also_send",
                "fn an_extra_layer_whose_stop_is_also_send(stop: &(dyn Fn() -> bool + Send + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_static",
                "fn an_extra_layer_whose_stop_is_static(stop: &(dyn Fn() -> bool + Sync + 'static)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_borrowed_mutably",
                "fn an_extra_layer_whose_stop_is_borrowed_mutably(stop: &mut (dyn Fn() -> bool + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_lives_for_the_whole_run",
                "fn an_extra_layer_whose_stop_lives_for_the_whole_run(stop: &'static (dyn Fn() -> bool + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_type_holds_a_comment",
                "fn an_extra_layer_whose_stop_type_holds_a_comment(stop: &(dyn Fn() -> bool /* 叫停 */ + Sync)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_boxed",
                "fn an_extra_layer_whose_stop_is_boxed(stop: Box<dyn Fn() -> bool + Sync>) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_names_sync_first",
                "fn an_extra_layer_whose_stop_names_sync_first(stop: &(dyn Sync + Fn() -> bool)) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_a_bare_trait_object",
                "fn an_extra_layer_whose_stop_is_a_bare_trait_object(stop: &dyn Fn() -> bool) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_an_impl",
                "fn an_extra_layer_whose_stop_is_an_impl(stop: impl Fn() -> bool) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_generic",
                "fn an_extra_layer_whose_stop_is_generic<Stop: Fn() -> bool + Sync>(stop: &Stop) -> bool \u{7b} false \u{7d}",
            ),
            (
                "StopRequestOfAnExtraLayer",
                "type StopRequestOfAnExtraLayer = dyn Fn() -> bool + Sync;\nfn an_extra_layer_whose_stop_is_an_alias(stop: &StopRequestOfAnExtraLayer) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_fn_mut",
                "fn an_extra_layer_whose_stop_is_fn_mut(stop: &mut dyn FnMut() -> bool) -> bool \u{7b} false \u{7d}",
            ),
            (
                "an_extra_layer_whose_stop_is_a_function_pointer",
                "fn an_extra_layer_whose_stop_is_a_function_pointer(stop: fn() -> bool) -> bool \u{7b} false \u{7d}",
            ),
        ] {
            let source_with_the_new_layer = format!("{source}\n{new_layer}\n");
            let found = ways_a_layer_handed_the_stop_request_could_drop_it(
                &library_files_with_one_replaced(ROOT_OF_THE_FAMILY, &source_with_the_new_layer),
            );
            assert!(
                found.iter().any(|finding| finding.contains(marker)),
                "叫停写成别的类型的新层 {marker} 找得到、红：{found:#?}"
            );
        }
    }

    /// （K4-2 A11、A12、B02）：注释与字符串里写的不算——签名行尾注释里的
    /// `fn` 不被当成层名，字符串、行尾注释里的 `let stop_request` 不算又绑一次，钉住那一行、短层函数体上的行尾注释不算改了那一句，都判绿。
    /// 扫崩溃放量模块族编进库的那几份（其中一份改过），不要卡。
    #[test]
    fn the_stop_request_scan_reads_layer_names_rebindings_and_pinned_lines_past_comments_and_strings(
    ) {
        let pinned_line = concat!(
            "    let stop_request = StopRequestRememberedInThisRun::new(stop_is_",
            "requested);\n"
        );
        // 收叫停的层签名里叫停参数上一行的行尾逗号，连同叫停参数那一行的开头：不点名上一行是哪个参数，
        // 只要还有一层的叫停参数单独占一行、上一行是别的参数，这一形就在；下面只改照文件表的次序头一份里的头一处
        let end_of_the_parameter_line_before_a_stop_parameter =
            concat!(",\n    stop_is_", "requested: &(dyn");
        let call_line_of_a_short_layer =
            "        &|block_sequence| selected_block_sequences.contains(&block_sequence),\n";
        for (anchor, replacement, form) in [
            (
                end_of_the_parameter_line_before_a_stop_parameter,
                end_of_the_parameter_line_before_a_stop_parameter.replacen(
                    ",\n",
                    ", // 判法单元指纹，fn 外面定\n",
                    1,
                ),
                "签名行尾注释里写 fn",
            ),
            (
                pinned_line,
                format!("{pinned_line}    let _quoted = \"let stop_request = the_never_stopping_request();\";\n"),
                "字符串里写 let stop_request",
            ),
            (
                pinned_line,
                format!("{pinned_line}    let _counted_once = 0; // let stop_request = 换掉记住的叫停\n"),
                "行尾注释里写 let stop_request",
            ),
            (
                pinned_line,
                pinned_line.replacen(";\n", "; // 记住的叫停\n", 1),
                "钉住那一行带行尾注释",
            ),
            (
                call_line_of_a_short_layer,
                call_line_of_a_short_layer.replacen(",\n", ", // 只判点了名的块\n", 1),
                "短层函数体上带行尾注释",
            ),
        ] {
            let (path, source) = first_library_file_holding(anchor).unwrap_or_else(|| {
                panic!("{form}：崩溃放量模块族编进库的那几份里有要改的那一处 {anchor}")
            });
            let with_comments_and_strings = source.replacen(anchor, &replacement, 1);
            assert_eq!(
                ways_a_layer_handed_the_stop_request_could_drop_it(
                    &library_files_with_one_replaced(path, &with_comments_and_strings),
                ),
                Vec::<String>::new(),
                "{form}，判绿"
            );
        }
    }

    /// 场景：扫崩溃放量模块族编进库的那几份（[`library_files_of_the_family`]，连同崩溃枚举用例的判段 `crash_state_enumeration.rs`
    /// 与卡池 `gpu_unit_checks.rs`）测试部分之外、去掉注释与空白的代码，看配了 GPU 的判段登记临时库之前问不问 GPU 批处理服务收不收
    /// （判段直接登记、写待判行、等结论，等不到的由统一任务服务照卡住停；行为用例在测试进程里连不上真服务，只能看代码）。
    /// 预期：第 ① 段镜像判定、判表、第 ② 段核对三个入口（`judge_my_blocks_on_gpu`、`judge_packed_tables_through_the_gpu_verification_server`、
    /// `verify_my_blocks_on_gpu`）各登记一次临时库（每一种写法在族里各份数到的处数加起来），族里一处都不起控制命令问 `serves`；
    /// 这些源文件都不经借卡通道借卡（借卡客户端的几个入口一个都不调）。
    /// 判别力（`crates/mutations.tsv`「判表那一族不登记临时库」）：判表那一族的登记那一句删掉，登记那一处少一处，这条红。
    #[test]
    fn a_judging_stage_registers_its_temporary_library_without_asking_the_gpu_verification_server_and_leases_no_card(
    ) {
        let code_without_comments_and_white_space =
            |outside_the_tests_of_the_file: String| -> String {
                let outside_the_tests = outside_the_tests_of_the_file + "\n";
                let lines: Vec<&str> = outside_the_tests.split('\n').collect();
                lines_read_past_comments_and_literals(&lines)
                    .iter()
                    .map(|read_line| read_line.without_comments.as_str())
                    .collect::<Vec<&str>>()
                    .join("\n")
                    .chars()
                    .filter(|character| !character.is_whitespace())
                    .collect()
            };
        let family = library_files_of_the_family();
        let judging_stage: Vec<(&str, String)> = family
            .iter()
            .zip(family_text_outside_the_tests(&family))
            .map(|((path, _source), outside_the_tests)| {
                (
                    *path,
                    code_without_comments_and_white_space(outside_the_tests),
                )
            })
            .collect();
        for (branch, written, count) in [
            (
                "三个入口各登记一次临时库",
                "lettemporary_library=temporary_library_of_the_judging_stage_registered_with_the_real_service(library,PendingRowFamilyOfTheGpuVerificationServer::",
                3,
            ),
            (
                "不起控制命令问服务收不收",
                ".arg(\"serves\")",
                0,
            ),
        ] {
            assert_eq!(
                judging_stage
                    .iter()
                    .map(|(_path, code)| code.matches(written).count())
                    .sum::<usize>(),
                count,
                "{branch}：恰好 {count} 处"
            );
        }
        for (file_name, code) in judging_stage.into_iter().chain([
            (
                "crash_state_enumeration.rs",
                code_without_comments_and_white_space(text_outside_the_tests_module(include_str!(
                    "crash_state_enumeration.rs"
                ))),
            ),
            (
                "gpu_unit_checks.rs",
                code_without_comments_and_white_space(text_outside_the_tests_module(include_str!(
                    "gpu_unit_checks.rs"
                ))),
            ),
        ]) {
            for lease_entry in [
                "cards_leased_by_this_process",
                "lease_cards_through_the_client",
                "cards_handed_over_or_leased_on_demand",
                "CardLeaseBorrower",
            ] {
                assert!(
                    !code.contains(lease_entry),
                    "{file_name} 测试模块之外调了借卡客户端的入口 {lease_entry}：验证包的判段与卡池不借卡"
                );
            }
        }
    }
}
