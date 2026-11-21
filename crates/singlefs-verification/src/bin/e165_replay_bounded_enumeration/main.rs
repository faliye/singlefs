//! E165（恢复重放有界穷举与变异杀伤界）：在小界内穷举输入，直接调 `singlefs_core::recovery::replay_journal`，
//! 照条款文字手写的判据逐条数违例（登记 `research/prompts/e165-preregistration.md`）。
//!
//! 被测对象只有 `replay_journal` 这一个函数的返回值；`records` 表由装置把内存盘上每个记录槽的字节过 `JournalRecord::parse` 建出来。
//! 装置调 core 的这几样（被测对象与它的真编码）：`replay_journal`、`JournalRecord::to_bytes` / `parse`、`back_chain_of`、
//! `crc32_castagnoli`、`record_offset`、`PoolReader`、`RootRecord`、`JournalRecord` 与它们字段的类型。判据一侧只用本地常量
//! 与生成历史时记下的真相（哪条记录属于哪次发布、每次发布的新根段、哪条在盘上），不调 core 的判定函数。
//!
//! 域（登记 5.2）：合法历史一族 L、L6、L9、U、X、E1、E2、R、C、C6；对抗一族 A、A6、AC。界由 `--bounds segment1|segment2` 给，
//! 不给是不指定范围：登记的两段界 `segment1`、`segment2` 逐段都判，各打各的配置行与判定行；`--only <域>`（可写几个，逗号分隔）、`--family legal|adversarial|all` 只跑一部分（指定范围）；
//! `--max-n <N>` 与另两族的 `--max-u <U>` 是缩域：给了的那一趟配置行末尾带 `narrowed_below_the_registered_bounds=<参数>`，不当正式产物。
//! 多盘域 M、M6、MC（登记 `research/prompts/e165-r2-prereg.md` 5.3，第三段）只在 `--profile r2 --family multi_disk` 时跑：每条记录在 0、1 号盘
//! 同一落点各一份、各自落或没落，「完整在盘上」取任一份，直接建表先 0 号盘再 1 号盘；`--family all` 仍只含合法历史与对抗两族。
//! 择根与重放合判一族 K、K9、KW、KL（登记 `research/prompts/e165-r3-prereg.md` 5.3 至 5.5；对抗根环 KL 照第 7 次重跑登记 `research/prompts/e165-r7-prereg.md` 1.3 照落点公式摆根，替掉 r3 的 KA）只在 `--profile r3 --family root_choice` 时跑：真 `choose_root`、`scan_journal`、`replay_journal` 三步串起来，判据 KJ0 至 KJ3 各报各的；`--family all` 不含它。带读故障的择根合判一族 KF、KF3、KF9、KFW、KG、KL（登记 `research/prompts/e165-r4-prereg.md` 5.3 至 5.6；KL 同上）只在 `--profile r4 --family root_choice_faults` 时跑：同样调真三步，多出每块盘两个系统配置槽、挂载开头的读故障、可写挂载读到的更新状态读不出那道闸（实现的取甲）与被判时的读故障，判据加 KJ1T，`--profile r4` 的其余族与 r3 档相同。
//! 线程数取环境变量 `E165_THREADS`，没设取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值）；
//! 内联单测用同一个取法；计数与线程数无关。
//! 产物是 `E7RESULT` 行，收尾 `name=done emitted=N`；耗时另打 `E7TIMING` 行，不进逐字比对。
//!
//! 同一份 4096 字节只过一次 `JournalRecord::parse`：生成一条记录时解一次、记下结果，之后每个落盘子集建表时取那一次的结果
//! （`parse` 是字节的纯函数）。
//!
//! 对抗域里序号 0 的记录自证校验和成立而 `JournalRecord::parse` 拒收（读到 0 当损坏），进不了表；装置逐态数出来报
//! （`states_holding_a_record_rejected_by_parse`）。停机条款 H4 照登记修订第 9 条：序号不是 0 的每条生成记录断言解得回原记录，
//! 对抗域字母表里序号 0 的记录断言 `parse` 拒收，没拒就停。
//!
//! 单状态重放（登记修订第 12 条）：`--replay-state <域>:<N>:<序号>` 只判枚举次序里的那一个状态，打一行 `name=state`
//! （这个状态让哪几个量加了一、实现的结局、状态的写法）；`--list-states <域>:<N>` 把这个 (域, N) 的每个状态照枚举次序各打一行。
//! 两种都单线程照工作单元的次序走，不跑自检与阳性对照，判据与正常跑是同一段代码。域名用产物里 `domain=` 的写法。
//!
//! 钉住的红例逐行重放：`--pinned-cases <表>` 读 `.claude/gate.d/e165-pinned-red-examples.tsv` 那种表（格式在它的文件头），
//! 按数据行的次序每行判一个状态、打一行 `name=state`（与 `--replay-state` 打的那一行是同一段代码）；按 ②③④ 找到状态之后
//! 拿它重算 ①（`sha256(<域>|<N>|<state= 的值，截到 ;first= 之前>)`），与表里不同、或有行找不到状态、或行本身不对
//! （不是二十一段、⑨ 起抄来的红例账那一行与 ②③④⑦⑧ 对不上），退 1。
//! 逐态行与 ⑦ 比不比得上由读这张表的一方（`scripts/experiment-apparatus/e165-pin-red-examples.py --replay`）判，装置不判。

use std::cell::Cell;
use std::collections::{btree_map, BTreeMap};
use std::env;
use std::fs;
use std::ops::Range;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
    TreeIdentifier,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::journal::{
    back_chain_of, record_offset, JournalRecord, JournalRecordOrdinalWithinPublish,
    JournalRecordPlaceInPublish, NamedUnit, FIRST_JOURNAL_COUNTER,
};
use singlefs_core::pointer::{BirthSequence, LocationEntry, NodePointer, PointerHead};
use singlefs_core::recovery::{
    choose_root, every_root_ring_slot, read_root_ring_slot, replay_journal, scan_journal,
    BadRootRingSlotReading, NamedUnitVerification, PoolReader, RecoveryFailure,
    RootRingSlotReading, CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR,
};
use singlefs_core::root_record::{RootRecord, UnmountMarker};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{
    journal_in_flight_record_limit, SystemConfiguration, SystemImmutableConfiguration,
    SystemImmutableSizes, SystemMutableConfiguration, SystemRuntimeConfiguration,
    SystemRuntimeQuantities,
};
use singlefs_verification::sha256::sha256_hexadecimal;

mod adversarial_family;
mod bounds_and_pinned_absolute_values;
mod constant_cross_checks_and_self_checks;
mod domains_and_legal_work_units;
mod judged_implementation_and_legal_state_judgement;
mod legal_states_and_record_tables;
mod memory_disk_and_legal_history_generator;
mod pinned_red_example_replay;
mod positive_controls;
mod replay_outcomes_and_specification;
mod result_rows;
mod ring_geometry_and_run_profiles;
mod root_choice_controls_rows_and_run;
mod root_choice_fault_cells_and_pinned_values;
mod root_choice_fault_command_line_and_run;
mod root_choice_fault_controls_and_rows;
mod root_choice_fault_enumeration_and_criteria;
mod root_choice_fault_judgement;
mod root_choice_fault_locations_and_system_slots;
mod root_choice_fault_world_and_read_views;
mod root_choice_history_and_disk;
mod root_choice_judgement_and_enumeration;
mod root_choice_pinned_values;
mod tallies_and_state_probes;
mod txg_order_split_and_control_readings;

use self::adversarial_family::*;
use self::bounds_and_pinned_absolute_values::*;
use self::constant_cross_checks_and_self_checks::*;
use self::domains_and_legal_work_units::*;
use self::judged_implementation_and_legal_state_judgement::*;
use self::legal_states_and_record_tables::*;
use self::memory_disk_and_legal_history_generator::*;
use self::pinned_red_example_replay::*;
use self::positive_controls::*;
use self::replay_outcomes_and_specification::*;
use self::result_rows::*;
use self::ring_geometry_and_run_profiles::*;
use self::root_choice_controls_rows_and_run::*;
use self::root_choice_fault_cells_and_pinned_values::*;
use self::root_choice_fault_command_line_and_run::*;
use self::root_choice_fault_controls_and_rows::*;
use self::root_choice_fault_enumeration_and_criteria::*;
use self::root_choice_fault_judgement::*;
use self::root_choice_fault_locations_and_system_slots::*;
use self::root_choice_fault_world_and_read_views::*;
use self::root_choice_history_and_disk::*;
use self::root_choice_judgement_and_enumeration::*;
use self::root_choice_pinned_values::*;
use self::tallies_and_state_probes::*;
use self::txg_order_split_and_control_readings::*;

// ── 本地常量：值抄自 kb，不从 `crates/` 引；[`cross_check_local_constants_against_the_implementation`] 逐个回比（登记第三节、停机条款 H2）。

/// journal 记录字节数。抄自 `.claude/kb/layout/01-first-txn.md`「journal 环，两盘互为镜像，记录 4096」那一行与
/// D23（journal 的角色与格式） 已定项 12「记录尺寸取 4 KiB」。
const LOCAL_JOURNAL_RECORD_BYTES: u64 = 4096;
/// 在飞上限的安全系数 F。抄自 D23（journal 的角色与格式） 已定项 18「在飞记录数上限的取值与语义」（「在飞记录数上限 = 环槽数 ÷ F」，默认环下 196608 ÷ 3）。
const LOCAL_JOURNAL_SAFETY_FACTOR: u64 = 3;
/// journal 环起点槽号。抄自 `.claude/kb/layout/01-first-txn.md`「journal 环，两盘互为镜像，记录 4096」那一行（环从 16 MiB 起：16 MiB ÷ 16384 = 槽 1024）。
const LOCAL_JOURNAL_RING_START_SLOT: u64 = 1024;
/// 落点粒度：一个槽 16384 字节。抄自 `.claude/kb/layout/01-first-txn.md`「journal 环，两盘互为镜像，记录 4096」那一行的 16 MiB 与槽 1024 之比。
const LOCAL_SLOT_BYTES: u64 = 16384;
/// 第一个 jsn 计数器。抄自 `crates/singlefs-checker/invariants/03-journal.md`「`| I-8.6 |`」那一行（I-8.6：落点式 `(计数器 − 1) mod 槽数 × 4096` 从计数器 1 起）。
const LOCAL_FIRST_JOURNAL_COUNTER: u64 = 1;
/// 索引节点单元字节数。抄自 `.claude/kb/decisions/08-核心索引结构.md` 里的 `format-const: NODE_BYTES = 16384`。
const LOCAL_INDEX_NODE_UNIT_BYTES: u64 = 16384;
/// 点名项里索引节点的类标签（码 2）。抄自 `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 17 的点名项字段表「类标签 1」与
/// D8（核心索引结构） 已定项 2「节点大小 16 KiB，钉成常量」那一条（码 2 是索引节点）。
const LOCAL_UNIT_CLASS_INDEX_NODE: u8 = 2;

/// 大环槽数（登记 5.1：64 槽，在飞上限 21）。
const BIG_RING_SLOTS: u64 = 64;
/// 两个小环（登记 5.1、5.2：6 槽上限 2，9 槽上限 3）。
const SMALL_RING_SIX_SLOTS: u64 = 6;
const SMALL_RING_NINE_SLOTS: u64 = 9;
/// 大环上历史的第一个计数器（登记 5.1：c_0 = 7，不回绕）；小环上取环槽数（历史的第二条记录落回槽 0）。
const BIG_RING_FIRST_HISTORY_COUNTER: u64 = 7;
/// 所选实例 g（登记 5.1）。
const CHOSEN_INSTANCE: u32 = 3;
/// 合法历史第一次发布的 txg（登记 5.1：t_0 = 100，E 域除外）。
const ORDINARY_FIRST_TXG: u64 = 100;
/// 记录头里那 8 字节 fsid（系统配置 fsid 的低 8 字节）；根记录的 16 字节 fsid 低 8 字节取同一个值。
const FILESYSTEM_IDENTIFIER_LOW_EIGHT_BYTES: u64 = 0x4531_3635_5245_504c;
const ROOT_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x4c, 0x50, 0x45, 0x52, 0x35, 0x36, 0x31, 0x45, 0x2d, 0x45, 0x31, 0x36, 0x35, 0x2d, 0x30, 0x31,
];
/// 历史的第一条记录前面没有记录：反向链取一个固定的非 0 值（登记 5.1；它的前一条不在盘上，不判）。
const BACK_CHAIN_WHEN_THE_PREVIOUS_SLOT_HOLDS_NOTHING: u32 = 0x5eed_0165;
/// 时间线之外的树 ID 水位从这里起（残留、阳性对照的假根）；时间线上的取 1000 + 发布序号。
const OFF_TIMELINE_WATERMARK_BASE: u64 = 9_000_000;
/// 内存盘每块盘的字节数：只回答 `device_size_in_bytes`，`replay_journal` 不读它。
const MEMORY_DISK_DEVICE_BYTES: u64 = 1 << 40;
/// 点名单元住的槽从这里起（单元区里随便取的一段，两块盘同槽）。
const NAMED_UNIT_FIRST_SLOT: u64 = 60_000;

const THREADS_ENVIRONMENT_VARIABLE: &str = "E165_THREADS";

thread_local! {
    /// 这条线程此刻是不是在 `replay_journal` 里：panic 钩子只在这时不打印（被变异的实现可以 panic 几百万次）。
    static IS_INSIDE_THE_REPLAY_CALL: Cell<bool> = const { Cell::new(false) };
}

// ── 计数

macro_rules! labelled_enumeration {
    ($(#[$attribute:meta])* enum $name:ident { $($variant:ident => $label:literal,)+ }) => {
        $(#[$attribute])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        enum $name {
            $($variant,)+
        }

        impl $name {
            const ALL: &'static [Self] = &[$(Self::$variant,)+];

            const fn label(self) -> &'static str {
                match self {
                    $(Self::$variant => $label,)+
                }
            }

            #[allow(dead_code, reason = "不是每个枚举都拿下标去索引计数数组")]
            fn index(self) -> usize {
                Self::ALL
                    .iter()
                    .position(|member| *member == self)
                    .expect("ALL 列了这个枚举的每一个成员")
            }
        }
    };
}

labelled_enumeration! {
    /// 每个 (域, N) 上数的量。违例类的量在第一次加一时记一个见证。
    enum Counter {
        States => "states",
        Panicked => "panicked",
        Failed => "failed",
        ReturnedRootViolations => "J1",
        RootPlusDiskViolations => "J2",
        InFlightLimitViolations => "J3",
        FarthestVersionViolations => "J4",
        FixedPointPremiseHoldsJudged => "j6_premise_holds_judged",
        FixedPointPremiseHoldsViolations => "J6_premise_holds",
        FixedPointPremiseFailsJudged => "j6_premise_fails_judged",
        FixedPointPremiseFailsViolations => "J6_premise_fails",
        FixedPointFirstCallFailed => "j6_first_call_failed",
        FixedPointThirdCallEqualsSecond => "j6_third_call_equals_second",
        FixedPointThirdCallDiffersOrIsMissing => "j6_third_call_differs_or_is_missing",
        Ia1Judged => "ia1_judged",
        Ia1Violations => "IA1",
        Ia2Judged => "ia2_judged",
        Ia2Violations => "IA2",
        AppliedAtLeastOne => "applied_ge1",
        AppliedAtLeastTwo => "applied_ge2",
        FarthestAtLeastOne => "farthest_ge1",
        FarthestAtLeastTwo => "farthest_ge2",
        AboveWaterOverLimit => "above_water_over_limit",
        AboveWaterAtLimit => "above_water_at_limit",
        SpecificationWholeTableMismatch => "S_whole_table",
        SpecificationGapContinuesMismatch => "S_consecutive_gap_continues",
        SpecificationGapStopsMismatch => "S_consecutive_gap_stops",
        ChosenRootIsALaterVersion => "chosen_root_is_a_later_version",
        OffTimelineLandings => "off_timeline_landings",
        LegalCrashStatesWhenTheRootWriteIsItsOwnSegment => "legal_crash_states_cut_own",
        LegalCrashStatesWhenTheRootWriteSharesTheRecordSegment => "legal_crash_states_cut_with",
        PairStatesWithoutAnInterval => "pair_states_without_an_interval",
        AsWholeTableViolations => "AS_whole_table",
        AsGapContinuesViolations => "AS_consecutive_gap_continues",
        AsGapStopsViolations => "AS_consecutive_gap_stops",
        MutualWholeTableAndGapContinues => "whole_table_vs_consecutive_gap_continues",
        MutualWholeTableAndGapStops => "whole_table_vs_consecutive_gap_stops",
        MutualGapContinuesAndGapStops => "consecutive_gap_continues_vs_consecutive_gap_stops",
        StructureViolations => "A_structure",
        LimitViolations => "A_limit",
        SpecificationExpectsFailure => "specification_expects_failure",
        StatesHoldingARecordRejectedByParse => "states_holding_a_record_rejected_by_parse",
        FixedPointQnPremiseHoldsJudged => "j6qn_premise_holds_judged",
        FixedPointQnPremiseHoldsViolations => "J6Qn_premise_holds",
        FixedPointQnPremiseFailsJudged => "j6qn_premise_fails_judged",
        FixedPointQnPremiseFailsViolations => "J6Qn_premise_fails",
        FirstReturnedRootTxgIsTheLargest => "first_returned_root_txg_is_the_largest",
        CopiedFieldsJudged => "j7_judged",
        J7aInstanceTableViolations => "J7a_instance_table",
        J7aAllocationRecordTreeRootViolations => "J7a_allocation_record_tree_root",
        J7aFilesystemIdentifierViolations => "J7a_filesystem_identifier",
        J7bJudged => "j7b_judged",
        J7bViolations => "J7b",
        J7cJudged => "j7c_judged",
        J7cViolations => "J7c",
        TablesDiffer => "tables_differ",
        RowsDiffer => "rows_differ",
        TablesDifferInReportedOrder => "tables_differ_in_reported_order",
        RowsDifferInReportedOrder => "rows_differ_in_reported_order",
        DirectTableHoldsARecord => "direct_table_holds_a_record",
        FullyLandedScanTableSizeChecked => "fully_landed_scan_table_size_checked",
        TablesKeepTheTxgOrder => "i812_states_hold",
        TablesBreakTheTxgOrder => "i812_states_fail",
        FixedPointQnPremiseHoldsTxgOrderHoldsJudged => "j6qn_premise_holds_i812_holds_judged",
        FixedPointQnPremiseHoldsTxgOrderHoldsViolations => "J6Qn_premise_holds_i812_holds",
        FixedPointQnPremiseHoldsTxgOrderFailsJudged => "j6qn_premise_holds_i812_fails_judged",
        FixedPointQnPremiseHoldsTxgOrderFailsViolations => "J6Qn_premise_holds_i812_fails",
        FixedPointQnPremiseFailsTxgOrderHoldsJudged => "j6qn_premise_fails_i812_holds_judged",
        FixedPointQnPremiseFailsTxgOrderHoldsViolations => "J6Qn_premise_fails_i812_holds",
        FixedPointQnPremiseFailsTxgOrderFailsJudged => "j6qn_premise_fails_i812_fails_judged",
        FixedPointQnPremiseFailsTxgOrderFailsViolations => "J6Qn_premise_fails_i812_fails",
        NewRootSegmentFieldsJudged => "j8_judged",
        J8aJudged => "j8a_judged",
        J8aInstanceTableViolations => "J8a_instance_table",
        J8aAllocationRecordTreeRootViolations => "J8a_allocation_record_tree_root",
        J8aUnmountMarkerViolations => "J8a_unmount_marker",
        J8bViolations => "J8b",
        J8cJudged => "j8c_judged",
        J8cViolations => "J8c",
        NewRootSegmentFieldsWithoutASource => "J8_without_a_source",
    }
}

labelled_enumeration! {
    /// J5 的状态对集合（登记 5.3 五个集合），另按「同段的那一段是哪一种」分出的行（登记第六节 Q6）。
    enum PairSet {
        Wide => "wide",
        CutOwnLegal => "cut_own_legal",
        CutOwnSameSegment => "cut_own_same_segment",
        CutOwnSameRecordSegment => "cut_own_same_segment_stopped_in_a_record_segment",
        CutOwnSameRootSegment => "cut_own_same_segment_stopped_in_a_root_segment",
        CutWithLegal => "cut_with_legal",
        CutWithSameSegment => "cut_with_same_segment",
        CutWithSameSegmentRootWriteDiffers => "cut_with_same_segment_root_write_differs",
        CutWithSameSegmentRootWriteAgrees => "cut_with_same_segment_root_write_agrees",
    }
}

labelled_enumeration! {
    /// 每个状态对集合上数的五个量：对数，今天的实现与理想恢复各自的 J5a、J5b 违例对数。
    enum PairColumn {
        Pairs => "pairs",
        TodayAppliedSetViolations => "today_j5a",
        TodayLandedVersionViolations => "today_j5b",
        IdealAppliedSetViolations => "ideal_j5a",
        IdealLandedVersionViolations => "ideal_j5b",
    }
}

labelled_enumeration! {
    /// 多盘 M、M6 一个状态两块盘落盘集合的关系（登记 r2 5.3「落后类别」；SQ17 把落后按方向拆两行）。
    enum LaggingClass {
        Equal => "equal",
        FirstDeviceBehind => "first_device_behind",
        SecondDeviceBehind => "second_device_behind",
        Crossed => "crossed",
    }
}

// ── 域（登记 5.2）

labelled_enumeration! {
    /// 登记 5.2 的域；U 域验证开、关各算一个。
    enum Domain {
        LegalBigRing => "L",
        LegalSmallRingSix => "L6",
        LegalSmallRingNine => "L9",
        NamedUnitsVerified => "U_verify_on",
        NamedUnitsNotVerified => "U_verify_off",
        MonotoneBigRing => "C",
        MonotoneSmallRingSix => "C6",
        AdversarialBigRing => "A",
        AdversarialSmallRingSix => "A6",
        AdversarialBackChain => "AC",
        CrossInstance => "X",
        TxgOneBelowTheLargest => "E1",
        TxgAtTheLargest => "E2",
        Residue => "R",
        MirroredBigRing => "M",
        MirroredSmallRingSix => "M6",
        MirroredMonotoneBigRing => "MC",
    }
}

// ── 阳性对照（登记 5.4）：判据表的每一行一条假结局，必须红

labelled_enumeration! {
    /// 判据表。加一条判据就多一个成员，[`control_readings_of`] 的穷举 `match` 逼着给它配一条对照。
    enum Criterion {
        LegalDefined => "J0",
        ReturnedRoot => "J1",
        RootPlusDisk => "J2",
        WithinTheInFlightLimit => "J3",
        FarthestVersion => "J4",
        J5aAppliedSet => "J5a",
        J5bLandedVersion => "J5b",
        J5aAndJ5bAreSeparable => "J5a_vs_J5b",
        FixedPoint => "J6",
        Ia1LargestTxgFails => "IA1",
        Ia2NoVerificationCounted => "IA2",
        AdversarialDefined => "A0",
        AsWholeTable => "AS_whole_table",
        AsGapContinues => "AS_consecutive_gap_continues",
        AsGapStops => "AS_consecutive_gap_stops",
        AMutual => "A_mutual",
        AStructure => "A_structure",
        ALimit => "A_limit",
        QnFixedPoint => "J6Qn",
        J7aInstanceTable => "J7a_instance_table",
        J7aAllocationRecordTreeRoot => "J7a_allocation_record_tree_root",
        J7aFilesystemIdentifier => "J7a_filesystem_identifier",
        J7bUnmountMarkerOfARebuiltRoot => "J7b",
        J7cUnmountMarkerWhenNothingIsApplied => "J7c",
        J8aFieldsFromTheNewRootSegment => "J8a",
        J8aUnmountMarkerFromTheNewRootSegment => "J8a_unmount_marker",
        J8bFilesystemIdentifierOfTheChosenRoot => "J8b",
        J8cNothingAppliedReturnsTheChosenRoot => "J8c",
        MultiDiskAnyCopyIsOnDisk => "multi_disk_any_copy",
        MultiDiskReturnedRoot => "J1_multi_disk",
        MultiDiskFarthestVersion => "J4_multi_disk",
        MultiDiskAppliedSetAndLandedVersion => "J5_multi_disk",
        TableComparisonSeesADifferentTable => "T_table",
        TableComparisonSeesADifferentRow => "T_row",
        QnFixedPointSplitByTxgOrder => "J6Qn_i812_split",
    }
}

labelled_enumeration! {
    /// 判据表（登记 5.5）；阳性对照那段循环遍历它。
    enum RootChoiceCriterion {
        Kj0Defined => "KJ0",
        Kj1NotEarlierThanTheLastConfirmedPublish => "KJ1",
        Kj2NotEarlierThanAnySelfVerifiedRoot => "KJ2",
        Kj3LandsOnTheIdealRecovery => "KJ3",
    }
}

labelled_enumeration! {
    /// 登记 5.4 的四个域：K、K9、KW 是合法历史；KL 是照落点公式摆根的对抗根环（第 7 次重跑登记 `research/prompts/e165-r7-prereg.md` 1.3，替掉 r3 的 KA）。
    enum RootChoiceDomain {
        BigRing => "K",
        SmallRingNine => "K9",
        WrappingRootRing => "KW",
        AdversarialRootRing => "KL",
    }
}

// ── 数的量与见证

labelled_enumeration! {
    /// 每个 (域, M, U) 格上数的量（登记 5.5 末段）。违例类的量在第一次加一时记一个见证。
    enum RootChoiceCounter {
        States => "states",
        ConfirmedStates => "confirmed_states",
        FirstSelfVerifiedRootIsNotTheNewest => "first_self_verified_root_is_not_the_newest",
        SameTxgDifferentInstanceRoots => "same_txg_different_instance_roots",
        HigherInstanceLowerTxgRoots => "higher_instance_lower_txg_roots",
        TwoOrMoreInstancesInTheRing => "two_or_more_instances",
        IdealRecoveryApplied => "ideal_recovery_applied",
        AboveNewestRootOverLimit => "above_newest_root_over_limit",
        AboveNewestRootAtLimit => "above_newest_root_at_limit",
        AboveChosenRootOverLimit => "above_chosen_root_over_limit",
        AboveChosenRootAtLimit => "above_chosen_root_at_limit",
        StatesWithoutASelfVerifiedRoot => "states_without_a_self_verified_root",
        PanickedStates => "panicked",
        ChoseNothingStates => "choose_root_returned_none",
        ReplayFailedStates => "replay_failed",
        ReplayAppliedStates => "replay_applied",
        Kj0Violations => "KJ0",
        Kj1Judged => "kj1_judged",
        Kj1Violations => "KJ1",
        Kj2Judged => "kj2_judged",
        Kj2Violations => "KJ2",
        Kj3Judged => "kj3_judged",
        Kj3Violations => "KJ3",
        SlotReadingsChecked => "slot_readings_checked",
        SlotReadingsDifferingFromTheTruth => "slot_readings_differing_from_the_truth",
    }
}

labelled_enumeration! {
    /// 登记 5.4 的六个域：KF、KF3、KF9、KFW 是带读故障的合法历史，KG 是把条款的两道保护拿掉的对照，KL 是与 r3 档同一个生成函数的对抗根环（第 7 次重跑登记 1.3、5.6 H2）。
    enum FaultDomain {
        GateOn => "KF",
        GateOnThreeMounts => "KF3",
        GateOnSmallRing => "KF9",
        GateOnWrappingRootRing => "KFW",
        GateOff => "KG",
        AdversarialRootRing => "KL",
    }
}

labelled_enumeration! {
    /// 一次挂载被拒在哪一步（登记 r5 修订 1 修1.五第 4 条「拒因分计」：闸那一栏 = 重读那一遍仍为真；取号那一刻的拒按 R2 的两道分；
    /// 取甲下取号那一刻只有一种拒，另记一栏，不进 R2 的三栏）。走了第二遍又在取号那一刻被拒的挂载记取号那一栏。
    enum MountRefusalReason {
        TheGateAfterTheOneReread => "gate",
        AcquisitionDeviceWithoutASelfVerifiedSlot => "acquisition_device_without_a_self_verified_slot",
        AcquisitionWitnessUndecidable => "acquisition_witness_undecidable",
        AcquisitionUnderTheImplementationChoice => "acquisition_under_the_implementation_choice",
    }
}

labelled_enumeration! {
    /// 被判的那一次恢复的读故障（登记 5.3 第 11 条）：无、根环里键最大的那个根槽读不出、最新实例的全部根读不出、某块盘不在。
    enum JudgedFault {
        Nothing => "none",
        NewestRoot => "RN",
        NewestInstanceRoots => "RI",
        DeviceZeroGone => "D0",
        DeviceOneGone => "D1",
    }
}

// ── 判据与计数（登记 5.5）

labelled_enumeration! {
    /// 判据表（登记 5.5）；阳性对照那段循环遍历它。KJ1T 是这一族多出来的一条（登记第一节）。
    enum FaultCriterion {
        Kj0Defined => "KJ0",
        Kj1NotEarlierThanTheLastConfirmedPublish => "KJ1",
        Kj1tDescendsFromTheLastConfirmedPublish => "KJ1T",
        Kj2NotEarlierThanAnyReadableRoot => "KJ2",
        Kj3LandsOnTheIdealRecovery => "KJ3",
    }
}

labelled_enumeration! {
    /// 本地写的择根序（登记第七节「本地择根序」那张表、9.1 六行替换文的比较式各写一份；落点 = 择到的根加理想重放，不调实现）。
    /// 次序与那张表的列一致：今天的序，v1823 至 v1826，v1822；最后一个是 KF1 用的「取键最小的根」。
    enum FaultLocalRule {
        Today => "today",
        IgnoresTheInstance => "v1823",
        FirstSelfVerifiedWins => "v1824",
        TieTakesTheLowerInstance => "v1825",
        InstanceFirstThenTxg => "v1826",
        TieTakesTheLater => "v1822",
        OldestWins => "oldest",
    }
}

// ── 命令行与主流程

struct RunSelection {
    /// 这一趟判的登记界，按次序逐段判（[`bounds_to_judge_of_the_option`]）。
    bounds_to_judge: Vec<BoundsTier>,
    only: Option<Vec<String>>,
    largest_size_cap: Option<usize>,
    families: Vec<Family>,
    family_label: &'static str,
}

const USAGE: &str = "用法：e165_replay_bounded_enumeration [--bounds segment1|segment2] [--family legal|adversarial|all|multi_disk|root_choice|root_choice_faults] [--only <域>[,<域>…]] [--max-n <N>] [--profile r1|r2|r3|r4|r5] [--marker-parity even|odd] [--table direct|scan|both] [--disk-order normal|reversed]（界不给是不指定范围，segment1、segment2 两段逐段都判；--max-n、--max-u 是缩域，给了的那一趟配置行带 narrowed_below_the_registered_bounds=、不当正式产物；档不给取 r1；奇偶、建表法、盘次序只在 r2、r3、r5 档有，不给分别取 even、direct、normal；这几个参数写在哪都行，单状态重放、逐态判定行、钉住的红例逐行重放同样认）；域名：L L6 L9 U C C6 A A6 AC X E1 E2 R M M6 MC（多盘三个域只在 --family multi_disk 时跑、要 --profile r2；all 只含合法历史与对抗两族；--family root_choice 只在 --profile r3 时跑，域 K K9 KW KL，认 --bounds、--only、--max-u；--family root_choice_faults 只在 --profile r4 时跑，域 KF KF3 KF9 KFW KG KL，认 --bounds、--only、--max-u、--gate r2（不给是取甲，r5 登记修订 1）；--profile r4 其余族与 r3 档相同；--profile r5 认 legal、adversarial、all、multi_disk，不认 root_choice、root_choice_faults，其余同 r3 档、只把 J7 换成 J8）。单状态重放：--replay-state <域>:<N>:<序号>；逐态判定行：--list-states <域>:<N>（这两种的域名用产物里 domain= 的写法，U 域是 U_verify_on、U_verify_off）；钉住的红例逐行重放：--pinned-cases <表>（格式见 .claude/gate.d/e165-pinned-red-examples.tsv 文件头）";

fn parse_the_command_line(arguments: &[String]) -> Result<RunSelection, String> {
    let mut bounds: Option<BoundsTier> = None;
    let mut only: Option<Vec<String>> = None;
    let mut largest_size_cap: Option<usize> = None;
    let mut families = vec![Family::LegalHistory, Family::Adversarial];
    let mut family_label = "all";
    let mut position = 0;
    while position < arguments.len() {
        let option = arguments[position].as_str();
        let value = arguments
            .get(position + 1)
            .ok_or_else(|| format!("{option} 后面缺一个值"))?
            .as_str();
        match option {
            "--bounds" => {
                bounds = Some(match value {
                    "segment1" => BoundsTier::SegmentOne,
                    "segment2" => BoundsTier::SegmentTwo,
                    unknown_tier => return Err(format!("不认识的界：{unknown_tier}")),
                });
            }
            "--family" => match value {
                "legal" => {
                    families = vec![Family::LegalHistory];
                    family_label = "legal";
                }
                "adversarial" => {
                    families = vec![Family::Adversarial];
                    family_label = "adversarial";
                }
                "multi_disk" => {
                    families = vec![Family::MultiDisk];
                    family_label = "multi_disk";
                }
                "all" => {}
                unknown_family => return Err(format!("不认识的族：{unknown_family}")),
            },
            "--only" => {
                let names: Vec<String> = value.split(',').map(str::to_string).collect();
                for name in &names {
                    if !Domain::ALL
                        .iter()
                        .any(|domain| domain.selection_name() == name)
                    {
                        return Err(format!("不认识的域：{name}"));
                    }
                }
                only = Some(names);
            }
            "--max-n" => {
                largest_size_cap = Some(
                    value
                        .parse::<usize>()
                        .map_err(|error| format!("--max-n 要一个非负整数：{error}"))?,
                );
            }
            unknown_option => return Err(format!("不认识的参数：{unknown_option}")),
        }
        position += 2;
    }
    Ok(RunSelection {
        bounds_to_judge: bounds_to_judge_of_the_option(bounds),
        only,
        largest_size_cap,
        families,
        family_label,
    })
}

/// 从参数里取出 `--profile` 与 `--marker-parity`（写在哪都行），交回档与其余参数（登记 r2 5.1、第八节）。
/// 档不给取 [`DEFAULT_PROFILE_LABEL`]；`--marker-parity` 只在 r2 档有。
fn split_off_the_profile_options(
    arguments: &[String],
) -> Result<(RunProfile, Vec<String>), String> {
    let mut profile_label: Option<String> = None;
    let mut marker_parity_label: Option<String> = None;
    let mut table_source_label: Option<String> = None;
    let mut disk_order_label: Option<String> = None;
    let mut remaining_arguments = Vec::new();
    let mut position = 0;
    while position < arguments.len() {
        let argument = &arguments[position];
        let destination = match argument.as_str() {
            "--profile" => Some(&mut profile_label),
            "--marker-parity" => Some(&mut marker_parity_label),
            "--table" => Some(&mut table_source_label),
            "--disk-order" => Some(&mut disk_order_label),
            _other_argument => None,
        };
        let Some(destination) = destination else {
            remaining_arguments.push(argument.clone());
            position += 1;
            continue;
        };
        let value = arguments
            .get(position + 1)
            .ok_or_else(|| format!("{argument} 后面缺一个值"))?;
        if destination.replace(value.clone()).is_some() {
            return Err(format!("{argument} 给了不止一次"));
        }
        position += 2;
    }
    let marker_parity = match marker_parity_label.as_deref() {
        None | Some("even") => MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
        Some("odd") => MarkerParity::OddVersionsWrittenByTheUnmountSequence,
        Some(unknown_parity) => {
            return Err(format!(
                "不认识的卸载记号奇偶：{unknown_parity}（只认 even、odd）"
            ))
        }
    };
    let table_source = match table_source_label.as_deref() {
        None | Some("direct") => RecordTableSource::Direct,
        Some("scan") => RecordTableSource::Scanned,
        Some("both") => RecordTableSource::Both,
        Some(unknown_table_source) => {
            return Err(format!(
                "不认识的建表法：{unknown_table_source}（只认 direct、scan、both）"
            ))
        }
    };
    let reported_disk_order = match disk_order_label.as_deref() {
        None | Some("normal") => ReportedDiskOrder::FirstDeviceBeforeSecond,
        Some("reversed") => ReportedDiskOrder::SecondDeviceBeforeFirst,
        Some(unknown_disk_order) => {
            return Err(format!(
                "不认识的盘次序：{unknown_disk_order}（只认 normal、reversed）"
            ))
        }
    };
    match profile_label.as_deref().unwrap_or(DEFAULT_PROFILE_LABEL) {
        "r1" if marker_parity_label.is_some() => {
            Err("--marker-parity 只在 --profile r2、r3 或 r5 时有".to_string())
        }
        "r1" if table_source_label.is_some() || disk_order_label.is_some() => {
            Err("--table 与 --disk-order 只在 --profile r2、r3 或 r5 时有".to_string())
        }
        "r1" => Ok((RunProfile::CONSTANT_FIELDS, remaining_arguments)),
        "r2" => Ok((
            RunProfile::varying_fields(marker_parity)
                .with_table_source(table_source)
                .with_reported_disk_order(reported_disk_order),
            remaining_arguments,
        )),
        "r3" | "r4" => Ok((
            RunProfile::txg_order_split(marker_parity)
                .with_table_source(table_source)
                .with_reported_disk_order(reported_disk_order),
            remaining_arguments,
        )),
        "r5" => Ok((
            RunProfile::root_fields_in_records(marker_parity)
                .with_table_source(table_source)
                .with_reported_disk_order(reported_disk_order),
            remaining_arguments,
        )),
        unknown_profile => Err(format!(
            "不认识的档：{unknown_profile}（只认 r1、r2、r3、r4、r5）"
        )),
    }
}

/// 配置行与收尾判决行里档的那几段：r1 档什么都不加（同 r1 时的装置逐字节）；r2 档的建表法与盘次序只在不是默认值时才写
/// （默认的 r2 档输出因此不因第四段加了这两个旋钮而多一个字）。
fn profile_fields_of_the_run_rows(run_profile: RunProfile) -> String {
    let table_source_field = match run_profile.table_source {
        RecordTableSource::Direct => String::new(),
        RecordTableSource::Scanned | RecordTableSource::Both => {
            format!(" table={}", run_profile.table_source.label())
        }
    };
    let disk_order_field = match run_profile.reported_disk_order {
        ReportedDiskOrder::FirstDeviceBeforeSecond => String::new(),
        ReportedDiskOrder::SecondDeviceBeforeFirst => {
            format!(" disk_order={}", run_profile.reported_disk_order.label())
        }
    };
    format!(
        "{}{table_source_field}{disk_order_field}",
        generator_and_criteria_fields_of_the_run_rows(run_profile)
    )
}

/// [`profile_fields_of_the_run_rows`] 里生成器与判据那两样的几段。
fn generator_and_criteria_fields_of_the_run_rows(run_profile: RunProfile) -> String {
    match (run_profile.generator, run_profile.criteria) {
        (
            FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
            CriteriaSet::RegisteredBeforeTheVaryingFieldsProfile,
        ) => String::new(),
        (
            FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
            CriteriaSet::WithQnFixedPointAndCopiedFields,
        ) => format!(" profile=r2 marker_parity={}", marker_parity.label()),
        (
            FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
            CriteriaSet::WithQnFixedPointAndCopiedFieldsSplitByTxgOrder,
        ) => format!(" profile=r3 marker_parity={}", marker_parity.label()),
        (
            FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
            CriteriaSet::WithQnFixedPointAndCopiedFields,
        ) => " profile=r1_generator_with_r2_criteria".to_string(),
        (
            FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
            CriteriaSet::WithQnFixedPointAndCopiedFieldsSplitByTxgOrder,
        ) => " profile=r1_generator_with_r3_criteria".to_string(),
        (
            FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
            CriteriaSet::RegisteredBeforeTheVaryingFieldsProfile,
        ) => format!(
            " profile=r2_generator_with_r1_criteria marker_parity={}",
            marker_parity.label()
        ),
        (
            FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity),
            CriteriaSet::WithQnFixedPointAndNewRootSegmentFieldsSplitByTxgOrder,
        ) => format!(" profile=r5 marker_parity={}", marker_parity.label()),
        (
            FieldsBeyondTheProjectionGenerator::SameForEveryVersion,
            CriteriaSet::WithQnFixedPointAndNewRootSegmentFieldsSplitByTxgOrder,
        ) => " profile=r1_generator_with_r5_criteria".to_string(),
    }
}

/// 线程数：`E165_THREADS`，没设取每台默认重活线程数（`SINGLEFS_THREAD_CAP`，再没设取 `.claude/heavy-thread-default` 的登记值）。
fn thread_count_from_the_environment() -> usize {
    thread_count_of_the_variable_values(
        env::var(THREADS_ENVIRONMENT_VARIABLE).ok().as_deref(),
        singlefs_verification::numeric_environment_variable::process_environment_variable(
            singlefs_verification::heavy_threads::THREAD_CAP_ENVIRONMENT_VARIABLE,
        )
        .as_deref(),
    )
}

/// [`thread_count_from_the_environment`] 的判法，两个变量的值由调用方给（`None` 是没设）：`E165_THREADS` 设了照它，
/// 没设经 `singlefs_verification::heavy_threads` 取每台默认重活线程数（`SINGLEFS_THREAD_CAP` 那一级在那里，这里不另留一份）。
///
/// # Panics
/// `E165_THREADS` 设了而不是正整数；落到默认时 `SINGLEFS_THREAD_CAP` 设了而不是正整数。
fn thread_count_of_the_variable_values(
    e165_threads_value: Option<&str>,
    thread_cap_value: Option<&str>,
) -> usize {
    match e165_threads_value {
        Some(text) => text
            .parse::<usize>()
            .ok()
            .filter(|thread_count| *thread_count > 0)
            .unwrap_or_else(|| {
                panic!("{THREADS_ENVIRONMENT_VARIABLE} 要是正整数，收到的是「{text}」")
            }),
        None => singlefs_verification::heavy_threads::default_heavy_threads_of_the_cap_value(
            thread_cap_value,
        )
        .get(),
    }
}

fn main() {
    let arguments_as_given: Vec<String> = env::args().skip(1).collect();
    let (run_profile, arguments) = match split_off_the_profile_options(&arguments_as_given) {
        Ok(split) => split,
        Err(problem) => {
            eprintln!("  ✗ {problem}");
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
    };
    if let [option, table_path] = arguments.as_slice() {
        if option == "--pinned-cases" {
            replay_every_pinned_red_example(table_path, run_profile);
            return;
        }
    }
    if arguments_name_the_root_choice_faults_family(&arguments) {
        let profile_is_the_faults_profile = arguments_as_given
            .windows(2)
            .any(|pair| pair[0] == "--profile" && pair[1] == "r4");
        if !profile_is_the_faults_profile {
            eprintln!(
                "  ✗ --family root_choice_faults 只在 --profile r4 时有（登记 r4 5.1：这一族只在 r4 档）"
            );
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
        run_the_root_choice_faults_family(&arguments);
        return;
    }
    if arguments_name_the_root_choice_family(&arguments) {
        if !run_profile.admits_the_root_choice_family() {
            eprintln!(
                "  ✗ --family root_choice 只在 --profile r3（或 r4）时有（登记 r3 5.1：这一族只在 r3 档；登记 r5 5.1：r5 档不认它）"
            );
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
        run_the_root_choice_family(&arguments, run_profile);
        return;
    }
    match parse_the_state_probe(&arguments) {
        Ok(Some((domain, _size, _probe)))
            if domain.family() == Family::MultiDisk && !run_profile.judges_j6qn() =>
        {
            eprintln!(
                "  ✗ 多盘域 {} 只在 --profile r2、r3、r5 时有",
                domain.label()
            );
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
        Ok(Some((domain, size, probe))) => {
            run_the_state_probe(domain, size, probe, run_profile);
            return;
        }
        Ok(None) => {}
        Err(problem) => {
            eprintln!("  ✗ {problem}");
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
    }
    let selection = match parse_the_command_line(&arguments) {
        Ok(selection) => selection,
        Err(problem) => {
            eprintln!("  ✗ {problem}");
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
    };
    if selection.families.contains(&Family::MultiDisk) && !run_profile.judges_j6qn() {
        eprintln!(
            "  ✗ --family multi_disk 只在 --profile r2、r3、r5 时有（登记 r2 5.1：多盘域不在 r1 档；登记 r5 5.1：r5 档认它）"
        );
        eprintln!("  → 怎么办：{USAGE}");
        std::process::exit(2);
    }
    install_the_panic_hook_that_stays_quiet_inside_the_replay_call();
    let outcomes: Vec<OutcomeOfTheBounds> = selection
        .bounds_to_judge
        .iter()
        .map(|bounds| run_the_selected_families_under_the_bounds(&selection, *bounds, run_profile))
        .collect();
    exit_when_some_bounds_ended_void_or_halted(&outcomes);
}

/// 合法历史、对抗、多盘几族在一段登记界下跑一趟（`main` 逐段调它）。
fn run_the_selected_families_under_the_bounds(
    selection: &RunSelection,
    bounds: BoundsTier,
    run_profile: RunProfile,
) -> OutcomeOfTheBounds {
    let thread_count = thread_count_from_the_environment();
    let run_started = Instant::now();
    let mut emitter = Emitter { emitted: 0 };
    let mut findings = RunFindings::default();
    emitter.emit_result_line(&format!(
        "name=config experiment=E165 bounds={} family={} only={} max_n={} chosen_instance={CHOSEN_INSTANCE} first_txg={ORDINARY_FIRST_TXG} big_ring_slots={BIG_RING_SLOTS} big_ring_first_counter={BIG_RING_FIRST_HISTORY_COUNTER} small_ring_slots={SMALL_RING_SIX_SLOTS},{SMALL_RING_NINE_SLOTS}{}{}",
        bounds.label(),
        selection.family_label,
        selection
            .only
            .as_ref()
            .map_or_else(|| "every_domain".to_string(), |names| names.join(",")),
        selection
            .largest_size_cap
            .map_or_else(|| "none".to_string(), |cap| cap.to_string()),
        profile_fields_of_the_run_rows(run_profile),
        narrowing_field_of_the_configuration_line(selection.largest_size_cap, "max_n"),
    ));

    // ① 常量回比断言与装置自检（断言不过当场 panic，进程以非 0 退出，一行结果都不算）；r2 档另跑 r2 那几项。
    let self_checks_of_the_profile: &[(&str, fn())] = if run_profile.judges_j6qn() {
        VARYING_FIELDS_SELF_CHECKS
    } else {
        &[]
    };
    // 多盘一族另跑多盘那几项（只在 --family multi_disk 时）。
    let self_checks_of_the_multi_disk_family: &[(&str, fn())] =
        if selection.families.contains(&Family::MultiDisk) {
            MULTI_DISK_SELF_CHECKS
        } else {
            &[]
        };
    // 记录表怎么建、盘次序有一个不是默认值时另跑这几项（只在 r2 档能设）。
    let self_checks_of_the_record_table_knobs: &[(&str, fn())] = if run_profile.table_source
        != RecordTableSource::Direct
        || run_profile.reported_disk_order != ReportedDiskOrder::FirstDeviceBeforeSecond
    {
        RECORD_TABLE_SELF_CHECKS
    } else {
        &[]
    };
    for (check_name, check) in SELF_CHECKS
        .iter()
        .chain(self_checks_of_the_profile)
        .chain(self_checks_of_the_multi_disk_family)
        .chain(self_checks_of_the_record_table_knobs)
    {
        check();
        emitter.emit_result_line(&format!("name=self_check check={check_name} passed=true"));
    }
    for ring in [
        RingGeometry::BIG,
        RingGeometry::SMALL_NINE,
        RingGeometry::SMALL_SIX,
    ] {
        emitter.emit_result_line(&format!(
            "name=in_flight_limit ring_slots={} local={} implementation={}",
            ring.ring_slots,
            ring.local_in_flight_limit(),
            journal_in_flight_record_limit(ring.ring_bytes())
        ));
    }

    // ② 阳性对照：遍历判据表的每一行；有一行没红，这一段作废（V2）。
    let controls_started = Instant::now();
    let control_readings =
        positive_control_readings(&selection.families, run_profile, thread_count);
    for reading in &control_readings {
        emitter.emit_result_line(&format!(
            "name=control criterion={} measured={} value={} must_see={} red_as_required={}",
            reading.criterion.label(),
            reading.measured_where,
            reading.value,
            reading.must_see,
            reading.is_red_as_required
        ));
        if !reading.is_red_as_required {
            findings.void_conditions.push(format!(
                "V2 判据 {} 的阳性对照没红：{} 量到 {}，必须 {}",
                reading.criterion.label(),
                reading.measured_where,
                reading.value,
                reading.must_see
            ));
        }
    }
    println!(
        "E7TIMING name=elapsed scope=positive_controls elapsed_seconds={:.3} threads={thread_count}",
        controls_started.elapsed().as_secs_f64()
    );

    // ③–⑥ 各域照登记 5.5 的次序：L、L 小环、U；C；A、A 小环、A 链；X、E、R。
    let mut total_states: u64 = 0;
    for domain in Domain::ALL {
        if !selection.families.contains(&domain.family()) {
            continue;
        }
        if selection.only.as_ref().is_some_and(|names| {
            !names
                .iter()
                .any(|name| name.as_str() == domain.selection_name())
        }) {
            continue;
        }
        let Some(registered_largest_size) = bounds.largest_size(*domain) else {
            continue;
        };
        let largest_size = selection
            .largest_size_cap
            .map_or(registered_largest_size, |cap| {
                cap.min(registered_largest_size)
            });
        if largest_size < domain.smallest_size() {
            continue;
        }
        let mut domain_total = Tally::new();
        for size in domain.smallest_size()..=largest_size {
            let size_started = Instant::now();
            let tally = tally_of_a_domain_size(
                *domain,
                size,
                Implementation::TodayReplayJournal,
                run_profile,
                thread_count,
            );
            assert_the_pinned_absolute_values(*domain, size, &tally);
            if run_profile.splits_j6qn_by_the_txg_order() && domain.family() == Family::Adversarial
            {
                assert_the_pinned_txg_order_state_counts(*domain, size, &tally);
                // 登记 r3 作废条款 TV6：拆开的四行对不回未拆的两行。
                if let Some(difference) = the_split_rows_differ_from_the_unsplit_rows(&tally) {
                    findings.void_conditions.push(format!(
                        "TV6 {} 域 n = {size} 拆开的 J6Qn 四行对不回未拆的两行：{difference}",
                        domain.label()
                    ));
                }
            }
            // 登记 r2 第七节甲类、SF9：每条记录在每块盘上都落了的那一态扫盘表的条数断言要真跑到过（一个都没跑到，这条断言是空的）。
            if run_profile.table_source != RecordTableSource::Direct
                && domain.family() != Family::Adversarial
                && tally.count_of(Counter::FullyLandedScanTableSizeChecked) == 0
            {
                findings.halt_conditions.push(format!(
                    "SF9 {} 域 N = {size} 没有一个「每条记录在每块盘上都落了」的态被断言过扫盘表的条数",
                    domain.label()
                ));
            }
            // 登记 r2 作废条款 SV6：两种建表法的表相同、判出来的行却不同（同一张表同一套判据不该有两种结局）。
            if run_profile.compares_the_two_record_tables()
                && tally.count_of(Counter::TablesDiffer) == 0
                && tally.count_of(Counter::RowsDiffer) > 0
            {
                findings.void_conditions.push(format!(
                    "SV6 {} 域 N = {size} 两种建表法的表逐态相同、判出来的行却有 {} 个态不同",
                    domain.label(),
                    tally.count_of(Counter::RowsDiffer)
                ));
            }
            emit_the_rows_of_a_domain_size(&mut emitter, *domain, size, &tally, run_profile);
            println!(
                "E7TIMING name=elapsed scope=domain domain={} n={size} states={} elapsed_seconds={:.3} threads={thread_count}",
                domain.label(),
                tally.count_of(Counter::States),
                size_started.elapsed().as_secs_f64()
            );
            domain_total.merge(&tally);
        }
        total_states += domain_total.count_of(Counter::States);
        emit_the_verdict_of_a_domain(
            &mut emitter,
            &mut findings,
            *domain,
            largest_size,
            &domain_total,
            run_profile,
        );
    }

    for condition in &findings.void_conditions {
        emitter.emit_result_line(&format!(
            "name=void condition={}",
            condition.replace(' ', "_")
        ));
    }
    for condition in &findings.halt_conditions {
        emitter.emit_result_line(&format!(
            "name=halt condition={}",
            condition.replace(' ', "_")
        ));
    }
    emitter.emit_result_line(&format!(
        "name=verdict scope=run bounds={} family={} states={total_states} self_checks_passed={} control_readings={} controls_not_red={} void_conditions={} halt_conditions={}{}",
        bounds.label(),
        selection.family_label,
        SELF_CHECKS.len()
            + self_checks_of_the_profile.len()
            + self_checks_of_the_multi_disk_family.len()
            + self_checks_of_the_record_table_knobs.len(),
        control_readings.len(),
        control_readings
            .iter()
            .filter(|reading| !reading.is_red_as_required)
            .count(),
        findings.void_conditions.len(),
        findings.halt_conditions.len(),
        profile_fields_of_the_run_rows(run_profile),
    ));
    println!(
        "E7TIMING name=elapsed scope=run states={total_states} elapsed_seconds={:.3} threads={thread_count}",
        run_started.elapsed().as_secs_f64()
    );
    emitter.done();
    OutcomeOfTheBounds::of_the_findings(&findings)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod root_choice_fault_tests;
