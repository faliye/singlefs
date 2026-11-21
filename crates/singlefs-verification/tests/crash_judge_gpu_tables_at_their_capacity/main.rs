//! 验证包模块：crash、crash_state_enumeration、crash_injection、crash_given_states、crash_amplification、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
//! GPU 判器按条数开的各张表在容量边界上：恰好装满的判得了、多一条的交 CPU，抬上限之前的边界上那几条流抬之后照样判得了。
//! 分配记录那两张表（恢复内核读一版分配记录树的 `MAX_ALLOCATION_RECORDS`，`crash_judge_common.wgsl`；扫描内核收一条根的分配记录的
//! `C_MAX_ALLOC_RECORDS`，`crash_judge_checker_common.wgsl`，最新根的已释放记录表 `C_MAX_RELEASED` 与它同宽）：容量 4096
//! （容器树旋钮压小的随机历史第 48、52 步起分配记录多于 2048 条，两个内核判不了），
//! 再抬到 8192（欠账 C894：整理夹着回退那一组比重的历史拉到 80、160 步时一版的分配记录多于 4096 条，最大 4728）。
//! 造法：新池新建文件之后逐次顺序写第一个文件，单元数长到 200 个、缩回 1 个为一轮（[`DATA_UNITS_OF_A_GROW_AND_SHRINK_ROUND`]），轮四遍或八遍，
//! 再长到若干个，最后被判的那次缩回 1 个。每个新写的单元每块盘一条分配记录，缩回去的单元那条记录照旧留着、改成已释放
//! （这段历史回退下界不抬，一条都不回收）；写表只录被判的那次发布（之前的录制流施加成起点镜像）。被判那一版的分配记录恰好 2048、2050、4098 条
//! （轮四遍再长到 56、57 个，轮八遍再长到 167 个；CPU 一侧现数的：四遍之后 1880 条、八遍之后 3712 条，每多长一个单元多两条）。
//! 两块盘互为镜像，每个落点每块盘一条记录，条数恒为偶数：上限之上最近造得出来的是加 2。
//! 2048、2050 那两条流每个状态 GPU 都判得了、对拍字（`crash_judge_gpu::CrossCheckWords`）与 CPU 逐状态相同（2050 条那一条钉「抬到 4096 判得到」：
//! 两张表改回 2048 时它红）；4098 那条流：两遍恢复有一遍落到被判那一版的状态，GPU 恢复内核判得了（8192 条装得下），恢复那几个对拍字与 CPU 相同
//! （恢复内核那张表改回 4096 时它红）；被判那次的根槽写落了的状态，池级 checker 记判不了（条目过多那一类）；两个内核都判得了的状态对拍字与 CPU 相同。
//! 这种不回收的流到 4096 条上下时，走读内核按单元记的几张表（引用集合、位置项校验和、I-3.5 落点表各 4096 条、走过的单元 2048 个、数据单元 1024 个，
//! CPU 一侧在全落的镜像上数到 4098、4098、4098、2049、1777）与分配记录一同到顶（走读内核那几张没跟着抬，欠账 C899），恰好 4096 条那一形 GPU 判不全，不立；
//! 随机历史那几张镜像回收过，那几张表各至多八百来条。8192 之上一条（8194 条）这种轮法造不出：轮十六遍那一形分配记录长到 6164 条就不再长
//! （CPU 一侧量的；txg 30 那一次写 200 个单元只多 36 条：新写的单元落回早先释放的槽，推的）。8194 条那条流换一种造法：新池新建文件之后
//! 顺序写 17 次各 200 个单元、再 201 个、再 144 个，被判那次缩回 1 个（[`sequential_writes_before_the_judged_publish_just_past_eight_thousand_one_hundred_and_ninety_two_records`]；
//! 每次整个文件重写，上一版的单元改成已释放、照旧留着，CPU 一侧现数的：前一版 8166 条、被判那一版 8194 条）。一个单元一条 journal 记录，
//! 这条流盘 0 上 3749 条记录写，两个内核的 journal 记录表 `MAX_RECORDS` 因此从 2048 抬到 4096（2048 时每个状态在扫环那一步就记判不了）。
//! 那条流：两遍恢复有一遍落到被判那一版的状态，GPU 恢复内核记判不了（条目过多）、交 CPU 判；两遍都落到前一版的状态，恢复内核判得了（8166 条装得下，
//! 恢复内核那张表改回 4096 时这些状态记判不了，它红），恢复那几个对拍字与 CPU 相同。
//! 回收过的流上一版分配记录多于 4096 条、引用不过 4096 条：整理夹着回退那一组比重拉到 160 步的一段，回退之后的镜像里有 27 张（4246 到 4728 条；
//! 前 80 步与拉到 80 步那一段逐步相同，那一段的 11 张在里面），每张作一个状态的给定状态表交 GPU 判器与 CPU 判器各判一遍，GPU 判得了、对拍字相同
//! （扫描内核那张表改回 4096 时它红）。I-5.4 与 I-3.16 排这几张的分配记录借的次序表是哪一段，卡上的判定钉不住（借回 `C_ROW_ORDER` 时溢出的那一截
//! 落进扫描内核此后不读的空段），落成扫描内核 naga 中间表示上的断言
//! （`the_scan_kernel_orders_the_allocation_records_in_the_allocation_record_row_order_at_both_call_sites`，不用卡）。
//! 已释放那张表：扫描内核先把最新根的分配记录收进分配记录那张表，再从里面挑已释放的，已释放的条数到不了分配记录的条数（这一版自己的单元总有
//! 未释放的记录），所以已释放多一条那一形走不到；`C_MAX_RELEASED` 直接取 `C_MAX_ALLOC_RECORDS`，站点 29 在每个输入上都走不到：这条推理在扫描内核的
//! naga 中间表示上逐步落成断言（`site_twenty_nine_of_the_scan_kernel_is_unreachable_because_released_records_are_picked_from_the_allocation_record_table`，
//! 不用卡），指站点 29「抬过上限报判不了」的变异是等价变异，留档在那一条。三条流上 CPU 判器每个状态照旧判、都不红（流是合法的）。
//! 扫描内核释放代高于有效回退下界 F 的已释放记录那张表（`C_MAX_RELEASED_ABOVE`，站点 34）从 2048 抬到与 `C_MAX_ALLOC_RECORDS` 同宽：
//! 这段历史 F = 0，被判那一版的已释放记录全数进这张表；轮四遍再长到 76 个单元的那条流分配记录 2088 条、已释放 2050 条（CPU 一侧现数的），
//! 每个状态 GPU 都判得了、对拍字与 CPU 逐状态相同，那张表改回 2048 时它红（容器树旋钮压小的随机历史第 53、55 步起已释放的多于 2048 条，就是这一处判不了）。
//! 同一条流也钉扫描内核 I-3.10 的去重表（`C_MAX_EXAMINED`，站点 31）：候选集里每条根与由记录施加出来的那几版，各版未释放的分配记录按（盘、槽、代）
//! 去重、整个状态累计一张；不回收的流上累计条数跟着最新那一版的分配记录条数长，这条流各版累计 2086 条（CPU 一侧现数的），那张表还是 2048 条时
//! 13 个状态都在站点 31 判不了（恢复照判、池级 checker 原因字 0x1f03：条目过多、站点 31）。抬到与 `C_MAX_ALLOC_RECORDS` 同宽之后每个状态判得了；
//! 2048、2050 那两条流各版累计 2046、2048 条，在旧的界上也装得下。此刻那张表满了之后不再记判不了、新来的记录不进表照判（I-3.10 是置位，
//! 同一条记录判两遍结论相同），站点 31 没有了，表多大只管少读几次单元头：这一条在扫描内核的 naga 中间表示上落成断言
//! （`judging_the_allocation_generations_never_leaves_the_state_undecidable_and_judges_on_once_the_examined_table_is_full`，不用卡）。
//! 合法的流造不出越过那张表的状态（推的，没造出反例）：回收要求释放代不高于回退下界，候选集里的版本都不低于它，各版未释放记录去重累计
//! 因此不多于最新那一版的分配记录条数，而一版多于 8192 条时分配记录那张表先满。
//! 表满之后照判那一判在卡上过的是一张不合法的镜像（`common_allocation_record_trees_copied_under_other_generations`）：一个 200 个单元的文件
//! 长好之后再发十九次只建一个空 inode 的发布，最新那条根之前的十九条老根各把分配记录树整棵照抄、改代、重封，同一批还在用的单元在每条老根里各算一份，
//! 各版未释放记录去重累计越过那张表、单版不过八百来条；这张镜像作一个状态的给定状态表交 GPU 判器与 CPU 判器各判一遍，GPU 判得了、对拍字相同（给定状态表那一路两边都不判 oracle，两遍 oracle 的位不比）。
//! journal 记录表 `MAX_RECORDS`（4096 条，两个内核都按它收环里的记录）这几条流 865、866、1780 条，8194 条那条流 3749 条（CPU 一侧数的盘 0 上的记录写）。
//! 同样造法、同样几半的还有实例表行那两张（扫描内核读一条根自己实例表全部行的 `C_R_MAX_ROWS`，`crash_judge_checker_common.wgsl`，与恢复内核读
//! 同一种表的 `MAX_ROWS` 同宽）：256 → 512（取号之后崩溃 369 次再挂载写 370 行那条流）→ 24576（取号之后崩溃 65 × 369 次再挂载十二次那个场景，
//! 最新根的实例表 23997 行、66 片）。取号之后崩溃 511 / 512 / 24575 / 24576 次再可写挂载，被录那次挂载写出的每条根自己那张实例表
//! 512 / 513 / 24576 / 24577 行。前三条流每个状态 GPU 都判得了、对拍字与 CPU 逐状态相同（24576 行那一条同时钉走读内核最新根实例表的行
//! `C_MAX_INSTANCE_ROWS` 从 1024 抬到 32768 与它的散列索引）；24577 那条流被判那条根落了的状态扫描内核在站点 32 记判不了、恢复落到那次挂载的根的状态
//! 恢复内核记条目过多，别的状态判得了、对拍字与 CPU 相同。
//! 主走读 inode → 对象出生代 `C_MAX_INODE_BIRTHS`（1024 → 16384，站点 13）、扫描内核一版的 inode 记录 `D_MAX_INODE_RECORDS`（1024 → 16384，站点 49）
//! 与 I-9.29 跨版本那张表 `D_MAX_OBJECT_BIRTHS`（2048 → 16384，站点 59）：一次建 2500 个空 inode、再建一个的那条流（各版合起来 2502 个号），
//! 每个状态 GPU 都判得了、对拍字与 CPU 逐状态相同；三张表各改回原来的条数时它红。
//! 判表自己的两个上限（枚举写表的宽 `crash_facts::MAXIMUM_ENUMERATED_WRITES`，持久掩码 `array<u32, PERSISTED_MASK_WORDS>`；一个落点的写数
//! `crash_judge_tables::MAXIMUM_WRITES_PER_LOCATION`，组合掩码 `COMBINATION_WORDS_OF_A_VERSION` 个字）：2048 → 8192、64 → 256
//! （随机历史越过原分配记录墙那一族全量那一趟建不出判表）。那一族的一段每步镜像建成一张给定状态表，
//! 枚举写表与盘 0 系统配置槽 0 号的写数都越过抬之前的上限：每个状态 GPU 都判得了、对拍字与 CPU 逐状态相同。
//! 要本机一张空闲显存够的 NVIDIA 独显：一张独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
//! （`gpu_unit_checks::gpu_test_start`）。卡经这个进程的卡池取，建好表之后按判器要的显存借
//! （`crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool`），等满了借不到报「本次未跑」。要 `gpu` 特性。
#![cfg(feature = "gpu")]

#[path = "../common/mod.rs"]
mod common;
#[path = "../common_allocation_record_trees_copied_under_other_generations/mod.rs"]
mod common_allocation_record_trees_copied_under_other_generations;
#[path = "../common_compaction_and_rollback_histories/mod.rs"]
mod common_compaction_and_rollback_histories;

use common::{
    build_pool, file_content, geometry, parameters,
    sequential_write_in_process_as_instance_one_and_advance_the_pool, BuiltPool,
    FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
};
use common_allocation_record_trees_copied_under_other_generations::{
    copy_the_allocation_record_tree_of_an_old_root_under_another_generation, FreeSlotsForTheCopies,
};
use common_compaction_and_rollback_histories::{
    compaction_and_rollback_history, is_a_rollback_step, run_the_history_keeping_the_step_images,
    seed_of_the_compaction_and_rollback_sampling, HistoryWithItsStepImages,
};
use singlefs_checker::image::{chosen_system_configurations, valid_roots};
use singlefs_checker::walk::allocation_record_count_under_root;
use singlefs_verification::crash::{
    crash_point_replay_plan_state_count, CrashPointReplaySegmentExpansion,
};
use singlefs_verification::crash_amplification::RecordedGivenStatesTable;
use singlefs_verification::crash_given_states::{
    judge_the_given_states_on_the_cpu, GivenStateConclusion,
};
use singlefs_verification::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
use singlefs_verification::crash_state_enumeration::{
    sample_crash_states_on_the_cpu, CrashPointReplayItem,
};

mod allocation_record_tables_at_capacity;
mod examined_table_full_and_long_histories;
mod inode_and_object_birth_tables_at_capacity;
mod instance_table_rows_at_capacity;
mod scan_kernel_sources_of_the_record_tables;

/// 这份测试文件的仓内路径：崩溃点重放取样过库的设点文件。
const THIS_FILE: &str =
    "crates/singlefs-verification/tests/crash_judge_gpu_tables_at_their_capacity.rs";

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};

use singlefs_core::admission::SpaceAdmission;

use singlefs_core::block_device::PhysicalBlockSizeInBytes;

use singlefs_core::make_filesystem::make_filesystem;

use singlefs_core::mount::mount_writable;

use singlefs_core::recovery::instance_table_chain_of_root;

use singlefs_core::transaction::{
    acquire_instance, publish_new_inodes, PoolWriter, TransactionOutput,
};

use singlefs_core::unit::data_unit_payload_capacity;

use singlefs_verification::crash_injection::HistoryAfterTheMakeFilesystem;

use singlefs_verification::crash_judge_conclusions::cross_check_words_without_the_oracle;

use singlefs_verification::crash_judge_conclusions::ClaimedVersionOfAGivenState;

use singlefs_verification::crash_judge_gpu::{
    checker_scan_kernel_source, cross_check_words_agree_in_the_cross_check,
    cross_check_words_of_cpu_judgement, cross_check_words_of_gpu_verdict, verdict_bits,
    CrossCheckWords, LandedRootWords, POOL_CHECKER_UNDECIDABLE_WORD, VERDICT_WORDS,
};

use singlefs_verification::crash_judge_tables::{
    build_judge_tables, build_judge_tables_for_the_given_persisted_sets,
    build_judge_tables_for_the_given_persisted_sets_of_the_recorded_writes_broken_at_the_seams_with_their_claimed_versions,
    JudgeFlow,
};

use singlefs_verification::fault_injection::SharedFaultPlan;

use singlefs_verification::gpu_unit_checks::gpu_test_start::{
    cpu_reference_table_read_back_from_the_pairing_library, gpu_card_pool_or_report_not_run,
    GpuCardPoolStartedAtTheFirstLease,
};

use singlefs_verification::history::{
    execute_history_handing_out_every_step_image, generate_history_with_weights, GenerationWeights,
    HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistorySeed, PerStepChecker,
};

use singlefs_verification::memory_pool::{
    writes_and_segments, writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion,
    RetainedWrite, SparseBlockDevice,
};

use singlefs_verification::segments::StepKind;

use singlefs_verification::{RecordingBlockDevice, RetainedOperation, SharedStream};

use std::collections::HashSet;

use wgpu::naga::{
    Arena, BinaryOperator, Expression, Function, Handle, Literal, LocalVariable, Module, Statement,
};

/// 段长不超过这个数的段逐个真子集展开（与 GPU 对拍那一份的默认展开上限同一档）。
const EXPAND_UP_TO: usize = 8;

/// 两张表此刻的上限：恢复内核一版的分配记录、扫描内核一条根的分配记录（已释放记录那张与它同宽）。
const RECORDS_THE_GPU_TABLES_HOLD: usize = 8192;

/// 两张表抬到 8192 之前装几条：4098 条那条流上恢复落到被判那一版的状态恢复内核判得了（恢复内核那张改回 4096 时它红）；
/// 整理夹着回退拉到 160 步那几张镜像一版多于它（扫描内核那张改回 4096 时它红）。
const RECORDS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE_TO_EIGHT_THOUSAND_ONE_HUNDRED_AND_NINETY_TWO:
    usize = 4096;

/// 两张表抬到 4096 之前装几条：恰好 2048 条那条流照旧判得了，2050 条那条流多于它（两张表改回 2048 时它红）。
const RECORDS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE_TO_FOUR_THOUSAND_AND_NINETY_SIX: usize = 2048;

/// 1024 条容量的已释放记录表（扫描内核）：2048 那条流上已释放的条数要多于它，改回去才红。
const RELEASED_RECORDS_THE_SCAN_KERNEL_HELD_BEFORE_THE_FIRST_RAISE: usize = 1024;

/// 抬到 8192 之前的上限之上最近造得出来的条数：两块盘互为镜像，一个落点每块盘一条记录，条数恒为偶数。
const RECORDS_JUST_PAST_THE_GPU_TABLES_BEFORE_THE_RAISE_TO_EIGHT_THOUSAND_ONE_HUNDRED_AND_NINETY_TWO: usize =
    RECORDS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE_TO_EIGHT_THOUSAND_ONE_HUNDRED_AND_NINETY_TWO + 2;

/// 抬到 4096 之前的上限之上最近造得出来的条数。
const RECORDS_JUST_PAST_THE_GPU_TABLES_BEFORE_THE_RAISE_TO_FOUR_THOUSAND_AND_NINETY_SIX: usize =
    RECORDS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE_TO_FOUR_THOUSAND_AND_NINETY_SIX + 2;

/// 此刻的上限之上最近造得出来的条数。
const RECORDS_JUST_PAST_THE_GPU_TABLES: usize = RECORDS_THE_GPU_TABLES_HOLD + 2;

/// 两个内核收环里 journal 记录的那张表（`crash_judge_common.wgsl` 的 `MAX_RECORDS`，满了记判不了、原因 2）此刻装几条。
const JOURNAL_RECORDS_THE_GPU_TABLES_HOLD: usize = 4096;

/// 那张表抬到 4096 之前装几条：8194 条那条流的记录多于它（改回去每个状态在扫环那一步判不了）。
const JOURNAL_RECORDS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE: usize = 2048;

/// 判不了的原因码：条目过多（`crash_judge_common.wgsl` 的 `UNDECIDABLE_TOO_MANY_ENTRIES`）。恢复内核哪张表满了（分配记录那张、
/// 读一版实例表的行那张）记在结论第 1 字，池级 checker 两个内核哪张表满了记在原因字的低 8 位。
const RECOVERY_UNDECIDABLE_TOO_MANY_ENTRIES: u32 = 3;

/// 池级 checker 判不了的原因字的低 8 位：条目过多那一类（高位是站点号）。
const POOL_CHECKER_UNDECIDABLE_REASON_MASK: u32 = 0xFF;

/// 一轮顺序写：长到 200 个单元、缩回 1 个（每长一次多 400 多条记录，缩回去的那些改成已释放、照旧留着）。
const DATA_UNITS_OF_A_GROW_AND_SHRINK_ROUND: [usize; 2] = [200, 1];

/// 2048 / 2050 那两条流被判那次之前轮几遍（之后 1880 条记录，CPU 一侧现数的）。
const GROW_AND_SHRINK_ROUNDS_OF_THE_FLOWS_AROUND_TWO_THOUSAND_AND_FORTY_EIGHT_RECORDS: usize = 4;

/// 4098 那条流被判那次之前轮几遍（之后 3712 条记录，同上）。
const GROW_AND_SHRINK_ROUNDS_OF_THE_FLOW_JUST_PAST_FOUR_THOUSAND_AND_NINETY_SIX_RECORDS: usize = 8;

/// 轮四遍之后再长到的单元数：被判那一版的分配记录恰好 [`RECORDS_THE_GPU_TABLES_HELD_BEFORE_THE_RAISE_TO_FOUR_THOUSAND_AND_NINETY_SIX`] 条（每个单元每块盘一条，多一个单元多两条）。
const DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_AT_TWO_THOUSAND_AND_FORTY_EIGHT_RECORDS: usize = 56;

/// 2050 那一形多长一个单元。
const DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_JUST_PAST_TWO_THOUSAND_AND_FORTY_EIGHT_RECORDS: usize =
    DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_AT_TWO_THOUSAND_AND_FORTY_EIGHT_RECORDS + 1;

/// 轮八遍之后再长到的单元数：被判那一版的分配记录 [`RECORDS_JUST_PAST_THE_GPU_TABLES_BEFORE_THE_RAISE_TO_EIGHT_THOUSAND_ONE_HUNDRED_AND_NINETY_TWO`] 条（CPU 一侧现数的：长到 167 个、再缩回 1 个之后 4098 条）。
const DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_JUST_PAST_FOUR_THOUSAND_AND_NINETY_SIX_RECORDS: usize =
    167;

/// 被判那次缩回的单元数。
const DATA_UNITS_OF_THE_JUDGED_PUBLISH: usize = 1;

/// 扫描内核释放代高于 F 的已释放记录那张表（`crash_judge_checker_common.wgsl` 的 `C_MAX_RELEASED_ABOVE`）抬之前装几条：
/// 这段历史 F = 0，被判那一版的已释放记录全数进这张表，要多于它，改回去才红。
const RELEASED_RECORDS_ABOVE_THE_FLOOR_THE_SCAN_KERNEL_HELD_BEFORE_THE_RAISE: usize = 2048;

/// 轮四遍之后再长到的单元数：被判那一版的已释放记录多于 [`RELEASED_RECORDS_ABOVE_THE_FLOOR_THE_SCAN_KERNEL_HELD_BEFORE_THE_RAISE`] 条
/// （CPU 一侧现数的：长到 76 个、再缩回 1 个之后分配记录 2088 条、已释放 2050 条；每多长一个单元两样各多两条，75 个时已释放恰好 2048 条）。
const DATA_UNITS_BEFORE_THE_JUDGED_PUBLISH_PAST_THE_OLD_RELEASED_ABOVE_THE_FLOOR_CAPACITY: usize =
    76;

/// 那一版的分配记录条数（CPU 一侧现数的）。
const RECORDS_OF_THE_VERSION_PAST_THE_OLD_RELEASED_ABOVE_THE_FLOOR_CAPACITY: usize = 2088;

/// 扫描内核 I-3.10 的去重表（`crash_judge_checker_common.wgsl` 的 `C_MAX_EXAMINED`；满了之后新来的记录不进表、照判）此刻装几条：与 `C_MAX_ALLOC_RECORDS` 同宽。
const EXAMINED_RECORDS_THE_SCAN_KERNEL_HOLDS: usize = RECORDS_THE_GPU_TABLES_HOLD;

/// 那张表抬到 4096 之前装几条：表满就记判不了（站点 31）的那时候，各版未释放记录去重累计多于它的流在表改回去时每个状态判不了。
const EXAMINED_RECORDS_THE_SCAN_KERNEL_HELD_BEFORE_THE_RAISE: usize = 2048;

/// 一条流各版未释放的分配记录按（盘、槽、代）去重累计的条数，落在 I-3.10 去重表抬之前的界的哪一边。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExaminedRecordsOfTheFlow {
    /// 不多于抬之前的 [`EXAMINED_RECORDS_THE_SCAN_KERNEL_HELD_BEFORE_THE_RAISE`] 条：这条流钉的不是那张表。
    WithinTheOldCapacityOfTheExaminedTable,
    /// 多于抬之前的那个界、不多于此刻的 [`EXAMINED_RECORDS_THE_SCAN_KERNEL_HOLDS`] 条：这条流钉那张表抬过。
    PastTheOldCapacityOfTheExaminedTable,
}

/// 一版的分配记录几条、其中已释放几条。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AllocationRecordCounts {
    every_record: usize,
    released: usize,
}

/// 一版未释放的分配记录按（盘、槽、代）记进 `examined`（同扫描内核 I-3.10 去重的键）。
fn note_the_unreleased_records(
    output: &TransactionOutput,
    examined: &mut HashSet<(DeviceIdentity, u64, CheckpointTxg)>,
) {
    examined.extend(
        output
            .allocation_records
            .iter()
            .filter(|record| !record.is_released)
            .map(|record| (record.device, record.slot.0, record.generation)),
    );
}

fn allocation_record_counts_of(output: &TransactionOutput) -> AllocationRecordCounts {
    AllocationRecordCounts {
        every_record: output.allocation_records.len(),
        released: output
            .allocation_records
            .iter()
            .filter(|record| record.is_released)
            .count(),
    }
}

/// 一条流：起点镜像（被判那次之前的录制流施加上去）、被判那次的写表与段、模型认的每一版，与每一版的分配记录条数。
struct RecordedFlow {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    judged_txg: CheckpointTxg,
    allocation_record_counts_of_each_version: Vec<(CheckpointTxg, AllocationRecordCounts)>,
    /// 新池新建文件那一版到被判那一版，各版未释放的分配记录按（盘、槽、代）去重累计（扫描内核 I-3.10 的去重表收的就是这一种）。
    unreleased_records_examined_across_the_versions: HashSet<(DeviceIdentity, u64, CheckpointTxg)>,
    /// mkfs 之后到被判那次发布为止盘 0 上的 journal 记录写（环不绕回、两块盘互为镜像：两个内核扫环收进记录表的就是这么多条）。
    journal_record_writes_on_device_zero_after_mkfs: usize,
}

impl RecordedFlow {
    fn allocation_record_counts_of_the_judged_version(&self) -> AllocationRecordCounts {
        self.allocation_record_counts_of_each_version
            .iter()
            .find(|(txg, _)| *txg == self.judged_txg)
            .map(|(_, counts)| *counts)
            .expect("被判那一版在每一版的条数表里")
    }

    /// 被判那一版之前那一版（txg 小一）的条数。
    fn allocation_record_counts_of_the_version_before_the_judged_one(
        &self,
    ) -> AllocationRecordCounts {
        self.allocation_record_counts_of_each_version
            .iter()
            .find(|(txg, _)| txg.0 + 1 == self.judged_txg.0)
            .map(|(_, counts)| *counts)
            .expect("被判那一版之前那一版在每一版的条数表里")
    }

    fn most_allocation_records_of_a_version_before_the_judged_one(&self) -> usize {
        self.allocation_record_counts_of_each_version
            .iter()
            .filter(|(txg, _)| *txg < self.judged_txg)
            .map(|(_, counts)| counts.every_record)
            .max()
            .expect("被判那一版之前还有版本")
    }
}

/// 第 `txg` 版文件的内容：`data_units` 个满载的数据单元，字节随 txg 换（相邻两版不同）。
fn content_of_the_version(data_units: usize, txg: CheckpointTxg) -> Vec<u8> {
    let shift = usize::try_from(txg.0 % 249).expect("小于 249");
    (0..data_units * data_unit_payload_capacity())
        .map(|index| u8::try_from((index + shift * 7) % 249).expect("小于 256"))
        .collect()
}

/// 在池上顺序写一次第一个文件（`data_units` 个单元），记下这一版的内容与分配记录条数。
fn publish_a_sequential_write(
    pool: &mut BuiltPool,
    data_units: usize,
    versions: &mut Vec<PublishedVersion>,
    counts: &mut Vec<(CheckpointTxg, AllocationRecordCounts)>,
    examined: &mut HashSet<(DeviceIdentity, u64, CheckpointTxg)>,
) {
    let txg = CheckpointTxg(pool.output.root.checkpoint_txg.0 + 1);
    let content = content_of_the_version(data_units, txg);
    let output = sequential_write_in_process_as_instance_one_and_advance_the_pool(pool, &content);
    assert_eq!(
        output.root.checkpoint_txg, txg,
        "一次顺序写一次发布、txg 加一"
    );
    assert_eq!(
        output.data_pointers.len(),
        data_units,
        "txg {} 那一版的单元数",
        txg.0
    );
    counts.push((txg, allocation_record_counts_of(&output)));
    note_the_unreleased_records(&output, examined);
    versions.push(PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: txg,
        content,
    });
}

/// 一段录制流里盘 0 上的 journal 记录写有几次。
fn journal_record_writes_on_device_zero(operations: &[RetainedOperation]) -> usize {
    let (writes, _segments) = writes_and_segments(operations, &geometry());
    writes
        .iter()
        .filter(|write| write.kind == StepKind::JournalRecord && write.device == DeviceIdentity(0))
        .count()
}

/// 新池新建文件（txg 3），照 [`DATA_UNITS_OF_A_GROW_AND_SHRINK_ROUND`] 顺序写 `grow_and_shrink_rounds` 遍，再长到 `data_units_before_the_judged_publish` 个单元，
/// 被判的是最后那次缩回 [`DATA_UNITS_OF_THE_JUDGED_PUBLISH`] 个单元的发布（[`record_the_flow_of_these_sequential_writes`]）。
fn record_the_flow(
    tag: &str,
    grow_and_shrink_rounds: usize,
    data_units_before_the_judged_publish: usize,
) -> RecordedFlow {
    let sequential_writes: Vec<usize> = std::iter::repeat_n(
        DATA_UNITS_OF_A_GROW_AND_SHRINK_ROUND,
        grow_and_shrink_rounds,
    )
    .flatten()
    .chain([data_units_before_the_judged_publish])
    .collect();
    record_the_flow_of_these_sequential_writes(tag, &sequential_writes)
}

/// 新池新建文件（txg 3），依次顺序写 `sequential_writes_before_the_judged_publish` 里每一项那么多个单元（一项一次发布），
/// 被判的是最后那次缩回 [`DATA_UNITS_OF_THE_JUDGED_PUBLISH`] 个单元的发布：起点镜像是 mkfs 到它之前的整段录制流施加出来的那一份，写表只录它。
fn record_the_flow_of_these_sequential_writes(
    tag: &str,
    sequential_writes_before_the_judged_publish: &[usize],
) -> RecordedFlow {
    let mut pool = build_pool(tag);
    let mut versions = vec![PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: pool.output.root.checkpoint_txg,
        content: file_content(),
    }];
    let mut counts = vec![(
        pool.output.root.checkpoint_txg,
        allocation_record_counts_of(&pool.output),
    )];
    let mut examined = HashSet::new();
    note_the_unreleased_records(&pool.output, &mut examined);
    // 迭代上界：sequential_writes_before_the_judged_publish.len() 次发布。
    for data_units in sequential_writes_before_the_judged_publish {
        publish_a_sequential_write(
            &mut pool,
            *data_units,
            &mut versions,
            &mut counts,
            &mut examined,
        );
    }
    let operations_before_the_judged_publish = pool.stream.operation_count();
    publish_a_sequential_write(
        &mut pool,
        DATA_UNITS_OF_THE_JUDGED_PUBLISH,
        &mut versions,
        &mut counts,
        &mut examined,
    );
    let judged_txg = pool.output.root.checkpoint_txg;
    let operations = pool.retained_operations();
    let mut base = pool.memory_pool_after_mkfs();
    base.apply(&operations[pool.mkfs_operation_count..operations_before_the_judged_publish]);
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_the_judged_publish..],
        &geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("被判那次发布有根槽写");
    RecordedFlow {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
        judged_txg,
        allocation_record_counts_of_each_version: counts,
        unreleased_records_examined_across_the_versions: examined,
        journal_record_writes_on_device_zero_after_mkfs: journal_record_writes_on_device_zero(
            &operations[pool.mkfs_operation_count..],
        ),
    }
}

/// 一个状态 CPU 那一侧判出的：对拍字、红项、两遍恢复有没有一遍落到被判那一版、被判那次的根槽写落没落。
struct CpuJudgementOfAState {
    cross_check_words: CrossCheckWords,
    red_items: Vec<String>,
    a_recovery_lands_on_the_judged_version: bool,
    judged_root_persisted: bool,
}

/// 每个状态两边各判一遍：GPU 结论字与 CPU 那一侧，按状态序号。卡借不到（等满了、或没有独显）交 `None`。
fn judge_every_state_on_both_sides(
    cards: GpuCardPoolStartedAtTheFirstLease,
    test_name: &str,
    flow: &RecordedFlow,
) -> Option<(Vec<[u32; VERDICT_WORDS]>, Vec<CpuJudgementOfAState>)> {
    let expansion = |_segment_index: usize, segment: &[usize]| {
        if segment.len() <= EXPAND_UP_TO {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    let tables = build_judge_tables(&JudgeFlow {
        base: &flow.base,
        writes: &flow.writes,
        segments: &flow.segments,
        judged_root_index: flow.judged_root_index,
        versions: &flow.versions,
        expansion: &expansion,
    })
    .expect("建得出表");
    let state_count =
        crash_point_replay_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
    assert_eq!(tables.state_count, state_count, "表与枚举的状态数相同");
    let gpu = judge_every_state_on_a_card_leased_from_the_pool(cards, test_name, &tables)?;
    let judged_txg = flow.judged_txg;
    let mut cpu: Vec<CpuJudgementOfAState> = Vec::new();
    let item_name = format!("tables_at_their_capacity::{test_name}::cpu_side");
    let judged_on_the_cpu = sample_crash_states_on_the_cpu(
        CrashPointReplayItem {
            name: &item_name,
            setting_file: THIS_FILE,
        },
        &flow.base,
        &flow.writes,
        &flow.segments,
        flow.judged_root_index,
        &flow.versions,
        &expansion,
        0..state_count,
        &mut |_ordinal, _image, judgement| {
            let lands_on_the_judged_version =
                |root: Option<(InstanceGeneration, CheckpointTxg)>| {
                    root.is_some_and(|(_, txg)| txg == judged_txg)
                };
            cpu.push(CpuJudgementOfAState {
                cross_check_words: cross_check_words_of_cpu_judgement(judgement),
                red_items: judgement.red_items(),
                a_recovery_lands_on_the_judged_version: lands_on_the_judged_version(
                    judgement.consulted.effective_root,
                ) || lands_on_the_judged_version(
                    judgement.ignored.effective_root,
                ),
                judged_root_persisted: judgement.root_persisted,
            });
        },
    )
    .judged()?;
    assert_eq!(judged_on_the_cpu, state_count, "CPU 判了每个状态");
    assert_eq!(cpu.len(), gpu.len(), "两边判的状态数相同");
    Some((gpu, cpu))
}

fn gpu_could_not_judge(gpu_words: &[u32; VERDICT_WORDS]) -> bool {
    gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
        || gpu_words[POOL_CHECKER_UNDECIDABLE_WORD] != 0
}

/// 流本身合法：CPU 判器每个状态都不红（这是「CPU 判器照旧判」那一半：GPU 判不了的状态 CPU 照样判完、判出来是绿的）。
fn assert_the_cpu_judge_finds_every_state_green(flow_name: &str, cpu: &[CpuJudgementOfAState]) {
    let red: Vec<String> = cpu
        .iter()
        .enumerate()
        .filter(|(_, judgement)| !judgement.red_items.is_empty())
        .take(8)
        .map(|(ordinal, judgement)| format!("状态 {ordinal}：{:?}", judgement.red_items))
        .collect();
    assert!(
        red.is_empty(),
        "流 {flow_name} 合法，CPU 判器每个状态都不该红（前几个）：\n{}",
        red.join("\n")
    );
}

/// 每一版的分配记录条数：被判那一版恰好 `judged_version_records` 条，之前每一版都不超过两张表的新上限（那几版 GPU 都判得了）。
fn assert_the_allocation_records_of_the_versions(
    flow_name: &str,
    flow: &RecordedFlow,
    judged_version_records: usize,
) {
    let judged = flow.allocation_record_counts_of_the_judged_version();
    assert_eq!(
        judged.every_record, judged_version_records,
        "流 {flow_name}：被判那一版（txg {}）的分配记录条数（每一版：{:?}）",
        flow.judged_txg.0, flow.allocation_record_counts_of_each_version
    );
    assert!(
        flow.most_allocation_records_of_a_version_before_the_judged_one()
            <= RECORDS_THE_GPU_TABLES_HOLD,
        "流 {flow_name}：被判那一版之前每一版的分配记录不超过 {RECORDS_THE_GPU_TABLES_HOLD} 条（每一版：{:?}）",
        flow.allocation_record_counts_of_each_version
    );
}

/// 每个状态 GPU 都判得了、对拍字与 CPU 逐状态相同（`cpu_cross_check_words` 与 `gpu` 同序）；不是的状态列出来（前几个）。
fn assert_every_state_judged_on_the_gpu_like_the_cpu_judge(
    flow_name: &str,
    gpu: &[[u32; VERDICT_WORDS]],
    cpu_cross_check_words: &[CrossCheckWords],
) {
    let mismatches: Vec<String> = cpu_cross_check_words
        .iter()
        .zip(gpu)
        .enumerate()
        .filter_map(|(ordinal, (cpu_words, gpu_words))| {
            let gpu_cross_check_words = cross_check_words_of_gpu_verdict(gpu_words);
            (gpu_could_not_judge(gpu_words)
                || !cross_check_words_agree_in_the_cross_check(&gpu_cross_check_words, cpu_words))
            .then(|| {
                format!(
                    "状态 {ordinal}：GPU 判不了 {}（恢复第 1 字 {:#x}、池级 checker 原因字 {:#x}）；GPU {gpu_cross_check_words:?}；CPU {cpu_words:?}",
                    gpu_could_not_judge(gpu_words),
                    gpu_words[1],
                    gpu_words[POOL_CHECKER_UNDECIDABLE_WORD],
                )
            })
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "流 {flow_name}：{} 个状态 GPU 判不了或与 CPU 不一致（前几个）：\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// 被判那一版的分配记录恰好 `judged_records` 条的那条流（[`record_the_flow`] 轮 `grow_and_shrink_rounds` 遍、再长到 `data_units_before_the_judged_publish` 个单元）：
/// 取卡之前在 CPU 上核条数、已释放的条数（多于 `released_more_than`）与各版未释放记录去重累计的条数（落在 `examined_records` 那一边），
/// 再在卡上逐状态判：每个状态 GPU 都判得了，对拍字与 CPU 逐状态相同；要有被判那次的根槽写落了、恢复落到那一版的状态（两个内核都读那一版的分配记录）。
fn assert_the_gpu_judge_judges_a_version_of_these_many_allocation_records_like_the_cpu_judge(
    test_name: &str,
    flow_name: &str,
    grow_and_shrink_rounds: usize,
    data_units_before_the_judged_publish: usize,
    judged_records: usize,
    released_more_than: usize,
    examined_records: ExaminedRecordsOfTheFlow,
) {
    let flow = record_the_flow(
        flow_name,
        grow_and_shrink_rounds,
        data_units_before_the_judged_publish,
    );
    assert_the_allocation_records_of_the_versions(flow_name, &flow, judged_records);
    let examined = flow.unreleased_records_examined_across_the_versions.len();
    let examined_records_of_the_flow = match examined {
        0..=EXAMINED_RECORDS_THE_SCAN_KERNEL_HELD_BEFORE_THE_RAISE => {
            ExaminedRecordsOfTheFlow::WithinTheOldCapacityOfTheExaminedTable
        }
        _ => ExaminedRecordsOfTheFlow::PastTheOldCapacityOfTheExaminedTable,
    };
    assert_eq!(
        examined_records_of_the_flow, examined_records,
        "流 {flow_name}：各版未释放的分配记录按（盘、槽、代）去重累计 {examined} 条，落在 I-3.10 去重表抬之前的 {EXAMINED_RECORDS_THE_SCAN_KERNEL_HELD_BEFORE_THE_RAISE} 条的哪一边"
    );
    assert!(
        examined <= EXAMINED_RECORDS_THE_SCAN_KERNEL_HOLDS,
        "流 {flow_name}：各版未释放记录去重累计 {examined} 条，不多于 I-3.10 去重表此刻的 {EXAMINED_RECORDS_THE_SCAN_KERNEL_HOLDS} 条"
    );
    let judged = flow.allocation_record_counts_of_the_judged_version();
    assert!(
        judged.released > released_more_than && judged.released <= RECORDS_THE_GPU_TABLES_HOLD,
        "流 {flow_name}：被判那一版已释放 {} 条，要多于 {released_more_than}、不多于 {RECORDS_THE_GPU_TABLES_HOLD}",
        judged.released
    );
    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
        return;
    };
    let Some((gpu, cpu)) = judge_every_state_on_both_sides(cards, test_name, &flow) else {
        return;
    };
    assert_the_cpu_judge_finds_every_state_green(flow_name, &cpu);
    assert!(
        cpu.iter().any(|judgement| {
            judgement.a_recovery_lands_on_the_judged_version && judgement.judged_root_persisted
        }),
        "流 {flow_name}：要有被判那次的根槽写落了、恢复落到那一版的状态（恢复内核读那一版 {judged_records} 条、扫描内核挑它已释放的 {} 条）",
        judged.released
    );
    let cpu_cross_check_words: Vec<CrossCheckWords> = cpu
        .iter()
        .map(|judgement| judgement.cross_check_words)
        .collect();
    assert_every_state_judged_on_the_gpu_like_the_cpu_judge(
        flow_name,
        &gpu,
        &cpu_cross_check_words,
    );
}

/// 在池上发一次建 `count` 个空 inode（号从记账水位起、实例 1），记下这一版（第一个文件不变）与它的分配记录条数。
fn publish_new_inodes_and_note_the_version(
    pool: &mut BuiltPool,
    count: u64,
    versions: &mut Vec<PublishedVersion>,
    counts: &mut Vec<(CheckpointTxg, AllocationRecordCounts)>,
    examined: &mut HashSet<(DeviceIdentity, u64, CheckpointTxg)>,
) {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let output = publish_new_inodes(
        &mut writer,
        &mut pool.allocator,
        &previous,
        count,
        FIXED_WRITE_TIME_SECONDS,
        InstanceGeneration(1),
    )
    .expect("建空 inode");
    counts.push((
        output.root.checkpoint_txg,
        allocation_record_counts_of(&output),
    ));
    note_the_unreleased_records(&output, examined);
    versions.push(PublishedVersion {
        instance: InstanceGeneration(1),
        checkpoint_txg: output.root.checkpoint_txg,
        content: file_content(),
    });
    pool.output = output;
}
