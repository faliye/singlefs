#!/usr/bin/env python3
"""E203（准入之前扫类的步代价）量的那一路：在拷来的实现上只插计数或只插计时，不改任何判定。

跑前登记 research/prompts/e203-preregistration.md 第五节「装置写在哪」第 1 条。

用法：
  python3 e203_admission_scan_cost_counters.py --build counts|timing --source <仓根> --into <工作目录> [--diff-out <文件>]
  python3 e203_admission_scan_cost_counters.py --check-anchors <树>      只核锚点、打每个锚点的行号，不写
  python3 e203_admission_scan_cost_counters.py --selftest [--histories <历史用例>]   用一棵小的假树证每个分支（不编实现）；
                                                                         另核历史用例里意图那次规模读数的取法（不给取本目录那一份）
  python3 e203_admission_scan_cost_counters.py --mutation-check <变异表>   逐条改坏、跑自证，没红的报 missed
  python3 e203_admission_scan_cost_counters.py --write-fake-source <新目录>   写一棵小的假仓根（入口脚本的自证用）
退出码：0 成；2 用法错或文件不在；3 有锚点命中不是恰好一次（一个字节不写，登记停机条款第 1 条）；4 自证有一格不对。

--build counts：只插计数与段切换，不插计时（模块 e203_scan_counts）。
--build timing：只插计时与段切换，不插计数（模块 e203_scan_timing），并去掉副本里 harness 的 lookup-work-counters 特性。
两种构建的副本里都有 `pub use … as e203_scan;`，历史用例只写 e203_scan::…，两边同一份源文件。
每个锚点在它那个函数体里必须恰好命中一次，否则退 3。
同一条命令被打断（内存上限杀、手动停）之后重起就是续跑：副本目录里先写开拷标记，拷完、改完才写完成标记（内容是输入键）。
再起时：完成标记的键与这一次的输入键相同，原样用这份副本；有开拷标记而没有完成标记（半拷）或键不同，清掉重拷；
非空而没有开拷标记（不是本脚本拷的）拒绝、不清，退 2。
弄坏开关（只给自证用，平常跑时不认；入口脚本的自证也经它用到）：
  E203_RUN_BREAK=copy-resume-off  不认完成标记与开拷标记，非空目录一律拒（回到不能续跑的写法）
      a_finished_copy_with_the_same_key_is_reused、a_finished_copy_whose_inputs_changed_is_cleared、a_copy_after_the_clearing_has_the_new_content、
      a_half_copy_is_cleared_and_copied_again、a_half_copy_ends_up_complete_after_the_second_try、
      a_copy_without_the_done_marker_is_not_trusted、a_begun_marker_cut_off_while_it_was_written_is_still_ours
      （失败行句首 name=selftest_case case=<这些名字> result=FAILED）
"""
import difflib
import hashlib
import os
import pathlib
import shutil
import sys

P = "crate::transaction::e203_scan::"
CORE = "crates/singlefs-core/src/"

# (CamelCase 行名, 计数名, 类型)；类型：scan 扫、copy 照抄、none 阴性对照、other 清单外归口
ROWS = [
    ("WholeAllocatorCopyBeforeTheJudgement", "whole_allocator_copy_before_the_judgement", "scan"),
    ("InodeLeafContainerCopy", "inode_leaf_container_copy", "copy"),
    ("InodeLeafWriterCopy", "inode_leaf_writer_copy", "copy"),
    ("InodeRecordLinearFind", "inode_record_linear_find", "scan"),
    ("FilesOtherThanTheFirstPlan", "files_other_than_the_first_plan", "scan"),
    ("FilesOtherThanTheFirstLowerNodeCopy", "files_other_than_the_first_lower_node_copy", "copy"),
    ("FileOtherThanTheFirstLinearFind", "file_other_than_the_first_linear_find", "scan"),
    ("InodesWithAnExtentEntrySet", "inodes_with_an_extent_entry_set", "scan"),
    ("ExtentUpperSegmentPlan", "extent_upper_segment_plan", "scan"),
    ("TombstoneContainerCopy", "tombstone_container_copy", "copy"),
    ("RetirementContainerCopy", "retirement_container_copy", "copy"),
    ("ExtentRolesRewrittenScan", "extent_roles_rewritten_scan", "scan"),
    ("RootHeaderLinearFind", "root_header_linear_find", "scan"),
    ("CheckpointCostAccountingTreePlan", "checkpoint_cost_accounting_tree_plan", "scan"),
    ("ClusterSegmentSum", "cluster_segment_sum", "scan"),
    ("AllocationRecordTreeNodeSet", "allocation_record_tree_node_set", "scan"),
    ("AllocationRecordTreePlan", "allocation_record_tree_plan", "scan"),
    ("LineageContainerCopy", "lineage_container_copy", "copy"),
    ("InodeNumberWatermarkSteps", "inode_number_watermark_steps", "scan"),
    ("DeadlistChangesIndexes", "deadlist_changes_indexes", "scan"),
    ("MultiLevelTreePlanning", "multi_level_tree_planning", "scan"),
    ("MappedUnitsOfContainerTrees", "mapped_units_of_container_trees", "scan"),
    ("MovedEntriesPathTailFilter", "moved_entries_path_tail_filter", "scan"),
    ("ReleaseJudgementVersionIndexes", "release_judgement_version_indexes", "scan"),
    ("WholeAllocatorCopyInASettlingRound", "whole_allocator_copy_in_a_settling_round", "scan"),
    ("AllocatorReleaseLinearFind", "allocator_release_linear_find", "scan"),
    ("UserDataSlotLinearScan", "user_data_slot_linear_scan", "scan"),
    ("RecordIntersectionFilter", "record_intersection_filter", "scan"),
    ("EmptySegmentLinearScan", "empty_segment_linear_scan", "scan"),
    ("FallbackSlotLinearScan", "fallback_slot_linear_scan", "scan"),
    ("RecordsOfEachLeaf", "records_of_each_leaf", "scan"),
    ("AccountingEntriesOfThisPublish", "accounting_entries_of_this_publish", "scan"),
    ("InstanceTableChain", "instance_table_chain", "scan"),
    ("UnitsOfAPublishToLand", "units_of_a_publish_to_land", "none"),
    ("MappedUnitsWholeTable", "mapped_units_whole_table", "scan"),
    ("UnlistedIndexOrPlanWork", "unlisted_index_or_plan_work", "other"),
]

# 模块模板：以 `//C ` 起头的行只在计数构建里留，以 `//T ` 起头的只在计时构建里留，别的两边都留。
MODULE_TEMPLATE = r'''
/// E203（准入之前扫类的步代价）只在副本里套上的读数：发布路径在两道准入之前每一行扫类的步数或独占计时，只读出、不进任何判定。
pub mod @MODULE@ {
    use std::cell::RefCell;
//T     use std::time::Instant;

    /// 一次发布在哪一段：登记读法第 1 条（调用方段、落点判之前、两道之间、准入之后）。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Segment {
        Outside,
        CallerBeforeTheJudgement,
        BeforeTheLandingJudgement,
        BetweenTheTwoJudgements,
        AfterTheJudgements,
    }
    const SEGMENTS: usize = 5;
    impl Segment {
        /// 产物里的段标记。
        #[must_use]
        pub fn tag(self) -> &'static str {
            match self {
                Segment::Outside => "outside",
                Segment::CallerBeforeTheJudgement => "caller",
                Segment::BeforeTheLandingJudgement => "before_landing",
                Segment::BetweenTheTwoJudgements => "between",
                Segment::AfterTheJudgements => "after",
            }
        }
    }
    const ALL_SEGMENTS: [Segment; SEGMENTS] = [
        Segment::Outside,
        Segment::CallerBeforeTheJudgement,
        Segment::BeforeTheLandingJudgement,
        Segment::BetweenTheTwoJudgements,
        Segment::AfterTheJudgements,
    ];

    /// 登记第三节清单里的一行（计数名）。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Row {
@ROW_VARIANTS@
    }
    pub const ROW_COUNT: usize = @ROW_COUNT@;
    const ALL_ROWS: [Row; ROW_COUNT] = [
@ROW_LIST@
    ];
    impl Row {
        #[must_use]
        pub fn name(self) -> &'static str {
            match self {
@ROW_NAMES@
            }
        }
    }

    /// 一段里一行的读数。
    #[derive(Clone, Debug)]
    pub struct RowReading {
        pub segment: &'static str,
        pub row: &'static str,
        pub calls: u64,
        pub steps: u64,
        /// 站点自己另读的表长之和：整份拷那两行是要拷的元素数；第七节 ③2 调用处读表长的那十行是全表扫那一部分每次调用那一刻的表长。
        pub scale: u64,
        /// `steps` 里属于走到命中为止那一部分（find、any、all、while 找第一个、contains）的步（修订 4）。
        pub steps_to_the_hit: u64,
        /// 走到命中为止那一部分每次调用那一刻要扫的表长之和（修订 4 的上界）。
        pub scale_to_the_hit: u64,
        pub exclusive_nanos: u64,
        /// 按身份建的索引上这一行归口到的查询次数，与每次查询的 ⌈log₂(索引项数 + 1)⌉ 之和（登记第五节 s19、s23、s36 的“查询次数 × lg”）。
        pub queries: u64,
        pub query_levels: u64,
    }
    /// 一条差量读数：哪一段、第几轮（固定点）、名字、值。
    #[derive(Clone, Debug)]
    pub struct DeltaReading {
        pub segment: &'static str,
        pub round: u32,
        pub key: &'static str,
        pub value: u64,
    }
    /// 一次发布准入之前读到的全部。
    #[derive(Clone, Debug, Default)]
    pub struct PublishRecord {
        /// 经调用方进门（有调用方段）；抬 F 预演直接进 `prepare_the_version_publish` 的是 false。
        pub had_caller_segment: bool,
        pub rows: Vec<RowReading>,
        /// 每段的段挂钟纳秒（计数构建里为空）。
        pub segment_nanos: Vec<(&'static str, u64)>,
        pub deltas: Vec<DeltaReading>,
        /// 固定点走了几轮。
        pub rounds: u32,
    }

    struct Open {
        had_caller_segment: bool,
        calls: Vec<u64>,
        steps: Vec<u64>,
        scale: Vec<u64>,
        steps_to_the_hit: Vec<u64>,
        scale_to_the_hit: Vec<u64>,
        nanos: Vec<u64>,
        queries: Vec<u64>,
        query_levels: Vec<u64>,
        segment_nanos: [u64; SEGMENTS],
        deltas: Vec<DeltaReading>,
        rounds: u32,
    }
    impl Open {
        fn new(had_caller_segment: bool) -> Self {
            let cells = SEGMENTS * ROW_COUNT;
            Self {
                had_caller_segment,
                calls: vec![0; cells],
                steps: vec![0; cells],
                scale: vec![0; cells],
                steps_to_the_hit: vec![0; cells],
                scale_to_the_hit: vec![0; cells],
                nanos: vec![0; cells],
                queries: vec![0; cells],
                query_levels: vec![0; cells],
                segment_nanos: [0; SEGMENTS],
                deltas: Vec::new(),
                rounds: 0,
            }
        }
        fn into_record(self) -> PublishRecord {
            let mut rows = Vec::new();
            for (segment_index, segment) in ALL_SEGMENTS.iter().enumerate() {
                for (row_index, row) in ALL_ROWS.iter().enumerate() {
                    let cell = segment_index * ROW_COUNT + row_index;
                    if self.calls[cell] + self.steps[cell] + self.scale[cell] + self.scale_to_the_hit[cell] + self.nanos[cell] + self.queries[cell] > 0 {
                        rows.push(RowReading {
                            segment: segment.tag(),
                            row: row.name(),
                            calls: self.calls[cell],
                            steps: self.steps[cell],
                            scale: self.scale[cell],
                            steps_to_the_hit: self.steps_to_the_hit[cell],
                            scale_to_the_hit: self.scale_to_the_hit[cell],
                            exclusive_nanos: self.nanos[cell],
                            queries: self.queries[cell],
                            query_levels: self.query_levels[cell],
                        });
                    }
                }
            }
            let segment_nanos = ALL_SEGMENTS
                .iter()
                .enumerate()
                .filter(|(index, _)| self.segment_nanos[*index] > 0)
                .map(|(index, segment)| (segment.tag(), self.segment_nanos[index]))
                .collect();
            PublishRecord {
                had_caller_segment: self.had_caller_segment,
                rows,
                segment_nanos,
                deltas: self.deltas,
                rounds: self.rounds,
            }
        }
    }

    struct Thread {
        segment: Segment,
        round: u32,
        open: Option<Open>,
        finished: Vec<PublishRecord>,
        attribution: Vec<Row>,
//T         since: Option<Instant>,
//T         stack: Vec<(Row, Instant, u64)>,
    }
    thread_local! {
        static THREAD: RefCell<Thread> = const {
            RefCell::new(Thread {
                segment: Segment::Outside,
                round: 0,
                open: None,
                finished: Vec::new(),
                attribution: Vec::new(),
//T                 since: None,
//T                 stack: Vec::new(),
            })
        };
    }

    /// 换段。离开 `Outside` 开一条记录（从调用方段进门的记 `had_caller_segment`），回到 `Outside` 收一条。
    pub fn segment(next: Segment) {
        THREAD.with(|cell| {
            let thread = &mut *cell.borrow_mut();
//T             let now = Instant::now();
//T             if let (Some(since), Some(open)) = (thread.since, thread.open.as_mut()) {
//T                 open.segment_nanos[thread.segment as usize] +=
//T                     u64::try_from(now.duration_since(since).as_nanos()).expect("纳秒装得进 u64");
//T             }
//T             thread.since = Some(now);
            if next == Segment::Outside {
                if let Some(open) = thread.open.take() {
                    thread.finished.push(open.into_record());
                }
            } else if thread.segment == Segment::Outside {
                thread.open = Some(Open::new(next == Segment::CallerBeforeTheJudgement));
            }
            thread.segment = next;
            thread.round = 0;
        });
    }

    /// 固定点每轮开头记一次。
    pub fn round_begin() {
        THREAD.with(|cell| {
            let thread = &mut *cell.borrow_mut();
            thread.round += 1;
            let round = thread.round;
            if let Some(open) = thread.open.as_mut() {
                open.rounds = open.rounds.max(round);
            }
        });
    }

    /// 交回到此刻收好的记录并清空；没收完的那条不交。
    #[must_use]
    pub fn take_finished() -> Vec<PublishRecord> {
        THREAD.with(|cell| std::mem::take(&mut cell.borrow_mut().finished))
    }

    thread_local! {
        /// 不量的那一段（历史第 1–4 步）停掉逐步的读数：为真时计数构建的调用、步数、表长、差量与规模读数一样不记，计时构建的守卫不计时。
        /// 段照旧换、记录照旧开与收（判了几次照数），只是记录里没有行。只读这一个线程局部的布尔，不借 `THREAD`。
        static READINGS_PAUSED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }
    /// 停（true）或恢复（false）这条线程上逐步的读数。
    pub fn pause_readings(paused: bool) {
        READINGS_PAUSED.with(|cell| cell.set(paused));
    }
    #[must_use]
    pub fn readings_paused() -> bool {
        READINGS_PAUSED.with(std::cell::Cell::get)
    }

    /// 归口：在它活着的这一段里，不属于某一行函数的步（码 2 规划、按身份建索引）记到哪一行。
    pub struct AttributionGuard;
    #[must_use]
    pub fn attribute(row: Row) -> AttributionGuard {
        THREAD.with(|cell| cell.borrow_mut().attribution.push(row));
        AttributionGuard
    }
    impl Drop for AttributionGuard {
        fn drop(&mut self) {
            THREAD.with(|cell| {
                cell.borrow_mut().attribution.pop();
            });
        }
    }
//C     fn attributed_row() -> Row {
//C         THREAD.with(|cell| cell.borrow().attribution.last().copied()).unwrap_or(Row::UnlistedIndexOrPlanWork)
//C     }

//C     fn with_open(change: impl FnOnce(&mut Open, usize, u32)) {
//C         if readings_paused() {
//C             return;
//C         }
//C         THREAD.with(|cell| {
//C             let thread = &mut *cell.borrow_mut();
//C             let segment = thread.segment as usize;
//C             let round = thread.round;
//C             if let Some(open) = thread.open.as_mut() {
//C                 change(open, segment, round);
//C             }
//C         });
//C     }
//C     /// 这一行在当前段里被调了一次。
//C     pub fn call(row: Row) {
//C         with_open(|open, segment, _| open.calls[segment * ROW_COUNT + row as usize] += 1);
//C     }
//C     /// 这一行在当前段里走了一步。
//C     pub fn step(row: Row) {
//C         steps(row, 1);
//C     }
//C     /// 这一行在当前段里走了 `count` 步。
//C     pub fn steps(row: Row, count: usize) {
//C         with_open(|open, segment, _| {
//C             open.steps[segment * ROW_COUNT + row as usize] += u64::try_from(count).expect("步数装得进 u64");
//C         });
//C     }
//C     /// 归口给当前挂着的行（没有挂着的记到清单外那一行）。
//C     pub fn call_attributed() {
//C         if readings_paused() {
//C             return;
//C         }
//C         call(attributed_row());
//C     }
//C     pub fn steps_attributed(count: usize) {
//C         if readings_paused() {
//C             return;
//C         }
//C         steps(attributed_row(), count);
//C     }
//C     /// 站点自己另读的循环长度（与走过的步独立，供登记第七节 ③2 对账）。
//C     pub fn scale(row: Row, count: usize) {
//C         with_open(|open, segment, _| {
//C             open.scale[segment * ROW_COUNT + row as usize] += u64::try_from(count).expect("长度装得进 u64");
//C         });
//C     }
//C     /// 站点在调用那一刻另读的、走到命中为止那一部分要扫的表长（修订 4 的上界；与走过的步独立）。
//C     pub fn scale_to_the_hit(row: Row, count: usize) {
//C         with_open(|open, segment, _| {
//C             open.scale_to_the_hit[segment * ROW_COUNT + row as usize] += u64::try_from(count).expect("长度装得进 u64");
//C         });
//C     }
//C     /// 这一行在当前段里走了 `count` 步，都属于走到命中为止那一部分：步数与这一部分的步数各加 `count`。
//C     pub fn steps_to_the_hit(row: Row, count: usize) {
//C         let count = u64::try_from(count).expect("步数装得进 u64");
//C         with_open(|open, segment, _| {
//C             let cell = segment * ROW_COUNT + row as usize;
//C             open.steps[cell] += count;
//C             open.steps_to_the_hit[cell] += count;
//C         });
//C     }
//C     /// 走到命中为止那一部分里走了一步。
//C     pub fn step_to_the_hit(row: Row) {
//C         steps_to_the_hit(row, 1);
//C     }
//C     /// 按身份建的索引上归口行的一次查询：索引有 `index_entries` 项，折半查至多 ⌈log₂(项数 + 1)⌉ 层。
//C     pub fn query_attributed(index_entries: u64) {
//C         if readings_paused() {
//C             return;
//C         }
//C         let row = attributed_row();
//C         with_open(|open, segment, _| {
//C             let cell = segment * ROW_COUNT + row as usize;
//C             open.queries[cell] += 1;
//C             open.query_levels[cell] += u64::from(u64::BITS - index_entries.leading_zeros());
//C         });
//C     }
//C     /// 线性 `contains` 走过的步：走到找到的那一个或走完（`Vec::contains` 同序），记在 `row` 走到命中为止的那一部分上。
//C     pub fn contains_counted<Item: PartialEq>(row: Row, haystack: &[Item], needle: &Item) -> bool {
//C         let walked = haystack.iter().position(|candidate| candidate == needle);
//C         steps_to_the_hit(row, walked.map_or(haystack.len(), |index| index + 1));
//C         walked.is_some()
//C     }
//C     /// 只在归口行正是 `row` 时记一条差量读数。
//C     pub fn delta_if_attributed(row: Row, key: &'static str, value: usize) {
//C         if !readings_paused() && attributed_row() == row {
//C             delta(key, value);
//C         }
//C     }
//C     /// 一条差量读数（B8 的输入），记在当前段当前轮。
//C     pub fn delta(key: &'static str, value: usize) {
//C         if readings_paused() {
//C             return;
//C         }
//C         let value = u64::try_from(value).expect("读数装得进 u64");
//C         THREAD.with(|cell| {
//C             let thread = &mut *cell.borrow_mut();
//C             let (segment, round) = (thread.segment.tag(), thread.round);
//C             if let Some(open) = thread.open.as_mut() {
//C                 open.deltas.push(DeltaReading { segment, round, key, value });
//C             }
//C         });
//C     }
//T     pub fn call(_row: Row) {}
//T     pub fn step(_row: Row) {}
//T     pub fn steps(_row: Row, _count: usize) {}
//T     pub fn call_attributed() {}
//T     pub fn steps_attributed(_count: usize) {}
//T     pub fn scale(_row: Row, _count: usize) {}
//T     pub fn scale_to_the_hit(_row: Row, _count: usize) {}
//T     pub fn steps_to_the_hit(_row: Row, _count: usize) {}
//T     pub fn step_to_the_hit(_row: Row) {}
//T     pub fn delta(_key: &'static str, _value: usize) {}
//T     pub fn query_attributed(_index_entries: u64) {}
//T     pub fn delta_if_attributed(_row: Row, _key: &'static str, _value: usize) {}

//T     /// 计时：守卫活着的这一段是这一行的独占计时（内层守卫的时间从外层减去）。
//T     pub struct TimerGuard {
//T         active: bool,
//T     }
//T     #[must_use]
//T     pub fn timer(row: Row) -> TimerGuard {
//T         if readings_paused() {
//T             return TimerGuard { active: false };
//T         }
//T         THREAD.with(|cell| {
//T             let thread = &mut *cell.borrow_mut();
//T             if thread.open.is_none() {
//T                 return TimerGuard { active: false };
//T             }
//T             thread.stack.push((row, Instant::now(), 0));
//T             TimerGuard { active: true }
//T         })
//T     }
//T     #[must_use]
//T     pub fn timer_attributed() -> TimerGuard {
//T         let row = THREAD
//T             .with(|cell| cell.borrow().attribution.last().copied())
//T             .unwrap_or(Row::UnlistedIndexOrPlanWork);
//T         timer(row)
//T     }
//T     impl Drop for TimerGuard {
//T         fn drop(&mut self) {
//T             if !self.active {
//T                 return;
//T             }
//T             THREAD.with(|cell| {
//T                 let thread = &mut *cell.borrow_mut();
//T                 let Some((row, started, children)) = thread.stack.pop() else {
//T                     return;
//T                 };
//T                 let elapsed = u64::try_from(started.elapsed().as_nanos()).expect("纳秒装得进 u64");
//T                 if let Some(parent) = thread.stack.last_mut() {
//T                     parent.2 += elapsed;
//T                 }
//T                 let segment = thread.segment as usize;
//T                 if let Some(open) = thread.open.as_mut() {
//T                     open.nanos[segment * ROW_COUNT + row as usize] += elapsed.saturating_sub(children);
//T                 }
//T             });
//T         }
//T     }
}
pub use @MODULE@ as e203_scan;
'''


def row(name):
    return f"{P}Row::{name}"


def seg(name):
    return f"{P}Segment::{name}"


def guard(name, indent="        "):
    return f"{indent}let _e203_timer = {P}timer({row(name)});\n"


def counted_call(name, indent="        "):
    return f"{indent}{P}call({row(name)});\n"


class Edit:
    """一处改动：file 里 head 起到 end 的函数体中，anchor 恰好命中一次；counts、timing 是两种构建各自插的文字（None = 这个构建不插）。"""

    def __init__(self, ident, file, head, end, anchor, mode, counts, timing):
        self.ident, self.file, self.head, self.end = ident, file, head, end
        self.anchor, self.mode, self.counts, self.timing = anchor, mode, counts, timing


FN_END = "\n}\n"
METHOD_END = "\n    }\n"
HEAD_AFTER_A_CHECK = "\nfn publish_version_of_trees_relocating_units_after_a_check<Device: BlockDevice, Refusal, Check>(\n"
HEAD_WRAPPER = "\npub fn prepare_the_version_publish<Device: BlockDevice>(\n"
HEAD_JUDGE = "\nfn judge_the_publish_before_any_read_or_write<Device: BlockDevice>(\n"
HEAD_SETTLE = "\nfn settle_the_allocation_record_tree(\n"
HEAD_ZERO_ROOT = "\npub fn plan_the_tree_after_this_publish_down_to_a_zero_root(\n"
T = CORE + "transaction.rs"
A = CORE + "allocator.rs"

WRAPPER_ANCHOR = (
    "    prepare_the_version_publish_relocating_units(\n        pool,\n        allocator,\n        plan,\n"
    "        previous,\n        trees,\n        release_checksum_check,\n        unmount_marker,\n"
    "        deletion_and_lineage,\n        &[],\n        &[],\n        &[],\n    )\n"
)
WRAPPER_NEW = (
    WRAPPER_ANCHOR.replace("    prepare_the_version", "    let e203_built = prepare_the_version", 1)[:-len("    )\n")]
    + "    );\n    " + P + "segment(" + seg("Outside") + ");\n    e203_built\n"
)
BOTH = lambda text: (text, text)  # noqa: E731

# 计时守卫的范围（登记第十二节修订 1：从函数级收严成那一步的语句）。
# body：守卫开在函数（或方法）体的第一句，因为这个函数体就是那一步（体里没有别的步，尾部是那一步的值）；
# block：守卫包在一对花括号里，只罩那一条语句；
# span：守卫开在那一步的第一句之前，由配对的 `<名>_end` 在最后一句之后 `drop`，函数里这个范围之外的语句不进该行的独占计时。
GUARD_SCOPE = {}
SPAN_NAMES = {}


def span(name, file, head, end, begin_anchor, end_anchor, row_name, indent, counts_begin="", counts_end="",
         timer=True, attribution=False, end_mode="before"):
    """一对 Edit：`<名>_begin` 在 begin_anchor 之前，`<名>_end` 在 end_anchor 之前（或之后）。
    计数构建只插 counts_begin / counts_end（和归口守卫）；计时构建只插计时守卫（和归口守卫）。"""
    timer_name, attribution_name = f"_e203_timer_{name}", f"_e203_attribution_{name}"
    attribute_begin = f"{indent}let {attribution_name} = {P}attribute({row(row_name)});\n" if attribution else ""
    attribute_end = f"{indent}drop({attribution_name});\n" if attribution else ""
    timer_begin = f"{indent}let {timer_name} = {P}timer({row(row_name)});\n" if timer else ""
    timer_end = f"{indent}drop({timer_name});\n" if timer else ""
    begin_counts = attribute_begin + counts_begin
    begin_timing = attribute_begin + timer_begin
    end_counts = counts_end + attribute_end
    end_timing = timer_end + attribute_end
    SPAN_NAMES[name] = (timer, attribution)
    if timer:
        GUARD_SCOPE[f"{name}_begin"] = "span"
    return [
        Edit(f"{name}_begin", file, head, end, begin_anchor, "before", begin_counts or None, begin_timing or None),
        Edit(f"{name}_end", file, head, end, end_anchor, end_mode, end_counts or None, end_timing or None),
    ]


# 登记第七节 ③2 的规模读数（修订 3 ②：式子只吃独立于被计数那一步的读数）：只在计数构建里插，计时构建不插。
# 入口：判之前读上一版（被量的发布扫的就是它）与这次的计划；出口：发完、换回 Outside 之前读这一次发出来的那一版。
# 读法只读字段长度与按号数记录，不调任何插了计数的函数（调了会把这一次的读数记进某一行的步数）。
# 每组：(包着这一组的那一行或 None, ((差量名, 表达式), ...))；差量记在当时那一段、轮 0。
SCALE_READINGS_AT_THE_ENTRY = (
    ("    if let Some(e203_version) = previous {\n", (
        ("entry_accounting_rows", "e203_version.accounting_entries.len()"),
        ("entry_watermark_rows_before_its_txg",
         "e203_version.accounting_entries.iter().filter(|entry| entry.statistic == STATISTIC_INODE_WATERMARK"
         " && entry.generation < e203_version.root.checkpoint_txg).count()"),
        ("entry_accounting_tree_keys", "e203_version.accounting_tree.shape.keys_in_order().len()"),
        ("entry_accounting_tree_nodes", "e203_version.accounting_tree.shape.nodes().len()"),
        ("entry_deadlist_tree_keys", "e203_version.deadlist_tree.shape.keys_in_order().len()"),
        ("entry_deadlist_tree_nodes", "e203_version.deadlist_tree.shape.nodes().len()"),
        ("entry_central_mapping_tree_keys", "e203_version.central_mapping_tree.shape.keys_in_order().len()"),
        ("entry_central_mapping_tree_nodes", "e203_version.central_mapping_tree.shape.nodes().len()"),
        ("entry_allocation_record_tree_nodes", "e203_version.allocation_record_tree.node_set().len()"),
        ("entry_units", "e203_version.units.len()"),
        ("entry_mapped_units", "e203_version.mapped_units.len()"),
        ("entry_files_other_than_the_first", "e203_version.files_other_than_the_first.len()"),
        ("entry_deadlist_entries", "e203_version.deadlist_entries.len()"),
        ("entry_kept_mapping_entries", "e203_version.mapping_entries_kept_for_the_snapshots.len()"),
        # 改 size 的那个 inode 的记录在叶容器摊平之后排第几（从 1 起；记录按号升序、叶之间不交叠，按「号更小的条数 + 1」数，不走 find）；
        # 不在就是全部记录数（find 走完）；这一步不改 size 记 0
        ("entry_position_of_the_inode_whose_size_changes",
         "deletion_and_lineage.step.inode_whose_size_changes().map_or(0, |e203_inode| {"
         " let e203_records = || e203_version.inode_leaf_containers.iter().flat_map(|container| container.contents.records.iter());"
         " if e203_records().any(|record| record.inode == e203_inode.0) {"
         " e203_records().filter(|record| record.inode < e203_inode.0).count() + 1 } else { e203_records().count() } })"),
    )),
    (None, (
        ("entry_instance_table_chain_replaced",
         "match &plan.instance_table { InstanceTablePlan::Rewrite(e203_rewrite) => e203_rewrite.replaced_chain.len(),"
         " InstanceTablePlan::Carry(_) => 0 }"),
        # s14 每块盘调一次、每次走完这一刻的聚簇段（修订 4）：落点判读的就是这个分配器，这一段里它不变
        ("entry_cluster_segments", "allocator.cluster_segments().len()"),
    )),
)
SCALE_READINGS_AT_THE_EXIT = (
    ("    if let Ok((_, e203_output)) = &built {\n", (
        ("exit_accounting_tree_keys", "e203_output.accounting_tree.shape.keys_in_order().len()"),
        ("exit_accounting_tree_nodes", "e203_output.accounting_tree.shape.nodes().len()"),
        ("exit_deadlist_tree_keys", "e203_output.deadlist_tree.shape.keys_in_order().len()"),
        ("exit_deadlist_tree_nodes", "e203_output.deadlist_tree.shape.nodes().len()"),
        ("exit_central_mapping_tree_keys", "e203_output.central_mapping_tree.shape.keys_in_order().len()"),
        ("exit_central_mapping_tree_nodes", "e203_output.central_mapping_tree.shape.nodes().len()"),
        ("exit_mapped_units", "e203_output.mapped_units.len()"),
        ("exit_allocation_record_tree_nodes", "e203_output.allocation_record_tree.node_set().len()"),
        ("exit_deadlist_entries", "e203_output.deadlist_entries.len()"),
        ("exit_container_tree_units",
         "e203_output.tombstone_containers.len() + e203_output.tombstone_tree_index_nodes.shape.positions_in_bump_order().len()"
         " + e203_output.lineage_containers.len() + e203_output.lineage_tree_index_nodes.shape.positions_in_bump_order().len()"
         " + e203_output.retirement_containers.len() + e203_output.retirement_tree_index_nodes.shape.positions_in_bump_order().len()"),
        ("exit_extent_tree_nodes",
         "e203_output.extent_tree.lower_nodes.len() + e203_output.extent_tree.upper_nodes.len()"
         " + e203_output.files_other_than_the_first.iter().map(|file| file.extent_lower_nodes.len()).sum::<usize>()"),
    )),
)


def readings_text(groups):
    """一组组规模读数插成的文字：包着的那一行下缩进 8 格，没有包的 4 格。
    整段包在「读数没停」里：不量的那一段（历史第 1–4 步）连读数的式子都不算（有的要走整张表），停没停见模块里的 `readings_paused`。"""
    text = f"    if !{P}readings_paused() {{\n"
    for opening, readings in groups:
        indent = "        " if opening else "    "
        text += opening or ""
        for key, expression in readings:
            text += f"{indent}{P}delta(\"{key}\", {expression});\n"
        text += "    }\n" if opening else ""
    return text + "    }\n"


def edits():
    e = []
    add = e.append
    # ---- 段切换与调用方（两种构建都插）
    add(Edit("segment_caller", T, HEAD_AFTER_A_CHECK, FN_END,
             "    refuse_while_a_publish_is_frozen(allocator)?;\n", "after",
             *BOTH(f"    {P}segment({seg('CallerBeforeTheJudgement')});\n")))
    # 出口读数排在换回 Outside 之前（同一个锚点前插，先插的在前）：记录还开着，读数进这一次发布的记录
    add(Edit("scale_readings_at_the_exit", T, HEAD_AFTER_A_CHECK, FN_END,
             "    let (writes, output) = match built {\n", "before", readings_text(SCALE_READINGS_AT_THE_EXIT), None))
    add(Edit("segment_outside_after_prepare", T, HEAD_AFTER_A_CHECK, FN_END,
             "    let (writes, output) = match built {\n", "before",
             *BOTH(f"    {P}segment({seg('Outside')});\n")))
    add(Edit("segment_outside_in_the_wrapper", T, HEAD_WRAPPER, FN_END, WRAPPER_ANCHOR, "replace",
             *BOTH(WRAPPER_NEW)))
    add(Edit("segment_landing_judgement", T, HEAD_JUDGE, FN_END,
             "    let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator\n", "before",
             *BOTH(f"    {P}segment({seg('BeforeTheLandingJudgement')});\n")))
    # 入口读数排在换进落点判之前那一段之后（同一个锚点前插，后插的在后）：上一版在被量的发布动它之前的样子
    add(Edit("scale_readings_at_the_entry", T, HEAD_JUDGE, FN_END,
             "    let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator\n", "before",
             readings_text(SCALE_READINGS_AT_THE_ENTRY), None))
    add(Edit("segment_between_the_judgements", T, HEAD_JUDGE, FN_END,
             "    let settled = settle_the_allocation_record_tree(\n", "before",
             *BOTH(f"    {P}segment({seg('BetweenTheTwoJudgements')});\n")))
    add(Edit("segment_after_the_judgements", T, HEAD_JUDGE, FN_END,
             "    Ok(PublishJudgedBeforeAnyReadOrWrite {\n", "before",
             *BOTH(f"    {P}segment({seg('AfterTheJudgements')});\n")))
    # ---- s1：调用方那一份整份拷
    # 整份拷的一步是被拷分配器里一个集合元素（登记读法第 3 条、第三节 s1 行），所以步数就是这一份要拷的元素数；
    # scale 另记同一个数，供第七节 ③1 拿它比 U 那一部分。只记 scale 不记步数时 Q1、Q2 与阳性对照读到的步数是 0。
    clone1 = "    let allocator_before_this_publish = allocator.clone();\n"
    add(Edit("s1_caller_copy", T, HEAD_AFTER_A_CHECK, FN_END, clone1, "replace",
             counted_call("WholeAllocatorCopyBeforeTheJudgement", "    ")
             + "    let e203_copied_elements = allocator.e203_whole_copy_elements();\n"
             + f"    {P}steps({row('WholeAllocatorCopyBeforeTheJudgement')}, e203_copied_elements);\n"
             + f"    {P}scale({row('WholeAllocatorCopyBeforeTheJudgement')}, e203_copied_elements);\n" + clone1,
             "    let allocator_before_this_publish = {\n"
             + f"        let _e203_timer = {P}timer({row('WholeAllocatorCopyBeforeTheJudgement')});\n"
             + "        allocator.clone()\n    };\n"))
    # ---- 固定点：轮、s24、s15
    add(Edit("settle_round_begin", T, HEAD_SETTLE, FN_END,
             "        let rewritten_among_the_nodes_after: BTreeSet<AllocationRecordTreeNode> =\n", "before",
             f"        {P}round_begin();\n", None))
    clone2 = "        let mut rehearsal = allocator.clone();\n"
    add(Edit("s24_rehearsal_copy", T, HEAD_SETTLE, FN_END, clone2, "replace",
             counted_call("WholeAllocatorCopyInASettlingRound")
             + "        let e203_copied_elements = allocator.e203_whole_copy_elements();\n"
             + f"        {P}steps({row('WholeAllocatorCopyInASettlingRound')}, e203_copied_elements);\n"
             + f"        {P}scale({row('WholeAllocatorCopyInASettlingRound')}, e203_copied_elements);\n" + clone2,
             "        let mut rehearsal = {\n"
             + f"            let _e203_timer = {P}timer({row('WholeAllocatorCopyInASettlingRound')});\n"
             + "            allocator.clone()\n        };\n"))
    add(Edit("s15_node_set", T, HEAD_SETTLE, FN_END,
             "    let mut rewritten: BTreeSet<AllocationRecordTreeNode> = BTreeSet::new();\n", "before",
             counted_call("AllocationRecordTreeNodeSet", "    ")
             + f"    {P}steps({row('AllocationRecordTreeNodeSet')}, 2 * previous_nodes.len());\n"
             + f"    {P}delta(\"previous_allocation_record_tree_nodes\", previous_nodes.len());\n"
             + f"    {P}delta(\"previous_allocation_records\", previous_records.len());\n", None))
    add(Edit("settle_round_work", T, HEAD_SETTLE, FN_END,
             "        let changed_by_the_release =\n            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());\n",
             "before",
             f"        {P}delta(\"round_placements_released\", release.len());\n"
             f"        {P}delta(\"round_roles_rewritten\", resolved.rewritten_roles.len());\n", None))
    changed_anchor = ("        let changed =\n            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());\n")
    add(Edit("settle_round_deltas", T, HEAD_SETTLE, FN_END, changed_anchor, "after",
             f"        {P}delta(\"allocation_nodes_changed\", changed.len());\n"
             f"        {P}delta(\"allocation_records_in_the_rehearsal\", rehearsal.records().len());\n"
             f"        {P}delta(\"placements_released\", release.len());\n"
             f"        {P}delta(\"roles_rewritten\", resolved.rewritten_roles.len());\n", None))
    # ---- 分配器里的扫类：s14、s25–s29
    add(Edit("s14_cluster_segment_sum", A, "    pub fn unblocked_slot_pairs_outside_the_cluster_segments(\n", METHOD_END,
             "        let pairs_inside_the_cluster_segments: u64 = cluster_segments\n", "before",
             counted_call("ClusterSegmentSum")
             + f"        {P}steps({row('ClusterSegmentSum')}, cluster_segments.len());\n",
             None))
    e.extend(span("s14_timing", A, "    pub fn unblocked_slot_pairs_outside_the_cluster_segments(\n", METHOD_END,
                  "        let pairs_inside_the_cluster_segments: u64 = cluster_segments\n",
                  "        self.unblocked_slot_pairs - pairs_inside_the_cluster_segments\n", "ClusterSegmentSum", "        "))
    find_a = "        self.records\n            .iter()\n            .find(|record| record.device == device && record.slot == slot)\n"
    add(Edit("s25_record_for", A, "    pub fn record_for(\n", METHOD_END, find_a, "replace",
             counted_call("AllocatorReleaseLinearFind")
             + f"        {P}scale_to_the_hit({row('AllocatorReleaseLinearFind')}, self.records.len());\n"
             + "        self.records\n            .iter()\n            .find(|record| {\n"
             + f"                {P}step_to_the_hit({row('AllocatorReleaseLinearFind')});\n"
             + "                record.device == device && record.slot == slot\n            })\n",
             guard("AllocatorReleaseLinearFind") + find_a))
    GUARD_SCOPE["s25_record_for"] = "body"
    # 修订 4：每块盘找一次，找之前读这一刻的记录数（上界那一半）
    add(Edit("s25_release_records_to_scan", A, "    pub fn release_leaving_the_record_allocated_on(\n", METHOD_END,
             "            let record = self\n                .records\n                .iter_mut()\n", "before",
             f"            {P}scale_to_the_hit({row('AllocatorReleaseLinearFind')}, self.records.len());\n", None))
    add(Edit("s25_release_find", A, "    pub fn release_leaving_the_record_allocated_on(\n", METHOD_END,
             "                .find(|record| record.device == device.device && record.slot == placement.slot)\n", "replace",
             "                .find(|record| {\n"
             + f"                    {P}step_to_the_hit({row('AllocatorReleaseLinearFind')});\n"
             + "                    record.device == device.device && record.slot == placement.slot\n                })\n",
             None))
    # 登记第三节 s25 一行的第二处查找（每块盘一次）：计数构建记一次调用，计时构建只罩这一句找（到 expect 为止）
    add(Edit("s25_release_call", A, "    pub fn release_leaving_the_record_allocated_on(\n", METHOD_END,
             "            let record = self\n                .records\n                .iter_mut()\n", "before",
             counted_call("AllocatorReleaseLinearFind", "            "), None))
    e.extend(span("s25_release_timing", A, "    pub fn release_leaving_the_record_allocated_on(\n", METHOD_END,
                  "            let record = self\n                .records\n                .iter_mut()\n",
                  "            assert!(\n                !record.is_released,\n", "AllocatorReleaseLinearFind", "            "))
    add(Edit("s26_user_slot_scan", A, "    pub fn lowest_user_data_slot_outside_the_region(\n", METHOD_END,
             "        while candidate + 1 < end {\n", "before",
             counted_call("UserDataSlotLinearScan")
             # 修订 4 的上界：这一刻单元区里的候选对数（每对一步）× (1 + 这一刻的聚簇段数，每个候选里 any 至多走完它们)
             + f"        {P}scale_to_the_hit({row('UserDataSlotLinearScan')}, usize::try_from(end.saturating_sub(candidate) / 2).expect(\"候选数装得进 usize\") * (1 + cluster_segments.len()));\n",
             None))
    e.extend(span("s26_timing", A, "    pub fn lowest_user_data_slot_outside_the_region(\n", METHOD_END,
                  "        while candidate + 1 < end {\n", "            candidate += 2;\n        }\n",
                  "UserDataSlotLinearScan", "        ", end_mode="after"))
    inside = ("            let inside_cluster_segment = cluster_segments.iter().any(|segment| {\n"
              "                candidate >= segment.0 && candidate < segment.0 + CLUSTER_SEGMENT_SLOTS\n            });\n")
    add(Edit("s26_candidate_steps", A, "    pub fn lowest_user_data_slot_outside_the_region(\n", METHOD_END, inside, "replace",
             f"            {P}step_to_the_hit({row('UserDataSlotLinearScan')});\n"
             "            let inside_cluster_segment = cluster_segments.iter().any(|segment| {\n"
             f"                {P}step_to_the_hit({row('UserDataSlotLinearScan')});\n"
             "                candidate >= segment.0 && candidate < segment.0 + CLUSTER_SEGMENT_SLOTS\n            });\n",
             None))
    add(Edit("s28_empty_segment_scan", A, "    pub fn lowest_empty_segment_outside_the_region(\n", METHOD_END,
             "        let per_segment = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect(\"64\");\n", "before",
             counted_call("EmptySegmentLinearScan"), None))
    add(Edit("s28_empty_segment_timer", A, "    pub fn lowest_empty_segment_outside_the_region(\n", METHOD_END,
             "        (0..full_segments)\n", "before", None, guard("EmptySegmentLinearScan")))
    GUARD_SCOPE["s28_empty_segment_timer"] = "body"
    # 修订 4 的上界：find 要扫的整段数，就是这一刻位图里的整段数（找之前读）
    add(Edit("s28_segments_to_scan", A, "    pub fn lowest_empty_segment_outside_the_region(\n", METHOD_END,
             "        (0..full_segments)\n", "before",
             f"        {P}scale_to_the_hit({row('EmptySegmentLinearScan')}, self.allocated.len() / per_segment);\n", None))
    add(Edit("s28_segment_steps", A, "    pub fn lowest_empty_segment_outside_the_region(\n", METHOD_END,
             "            .find(|segment| {\n                self.used_per_segment[*segment] == 0\n", "replace",
             "            .find(|segment| {\n" + f"                {P}step_to_the_hit({row('EmptySegmentLinearScan')});\n"
             + "                self.used_per_segment[*segment] == 0\n", None))
    add(Edit("s29_fallback_scan", A, "    pub fn lowest_commit_generated_fallback_slot_outside_the_region(\n", METHOD_END,
             "        while candidate + footprint.slots() <= unit_area_end {\n", "before",
             counted_call("FallbackSlotLinearScan")
             # 修订 4 的上界：这一刻单元区里按对齐步长排得下的候选数（每个一步）× (1 + 跨度槽数，每个候选里 all 至多走完跨度)
             + f"        {P}scale_to_the_hit({row('FallbackSlotLinearScan')}, usize::try_from(unit_area_end.checked_sub(candidate + footprint.slots()).map_or(0, |e203_room| e203_room / footprint.alignment_in_slots() + 1) * (1 + footprint.slots())).expect(\"上界装得进 usize\"));\n",
             None))
    e.extend(span("s29_timing", A, "    pub fn lowest_commit_generated_fallback_slot_outside_the_region(\n", METHOD_END,
                  "        while candidate + footprint.slots() <= unit_area_end {\n",
                  "            candidate += footprint.alignment_in_slots();\n        }\n",
                  "FallbackSlotLinearScan", "        ", end_mode="after"))
    add(Edit("s29_candidate_steps", A, "    pub fn lowest_commit_generated_fallback_slot_outside_the_region(\n", METHOD_END,
             "            let is_whole_span_unblocked = (candidate..candidate + footprint.slots()).all(|slot| {\n", "replace",
             f"            {P}step_to_the_hit({row('FallbackSlotLinearScan')});\n"
             "            let is_whole_span_unblocked = (candidate..candidate + footprint.slots()).all(|slot| {\n"
             f"                {P}step_to_the_hit({row('FallbackSlotLinearScan')});\n", None))
    add(Edit("s27_record_intersection", A, "\nfn make_room_for_record_on_device(\n", FN_END,
             "    let intersecting_record_slots: Vec<SlotNumber> = records\n        .iter()\n        .filter(|record| {\n",
             "replace",
             counted_call("RecordIntersectionFilter", "    ")
             # 修订 4：filter 走完这一刻的记录（全表扫那一部分，逐格相等）
             + f"    {P}scale({row('RecordIntersectionFilter')}, records.len());\n"
             + "    let intersecting_record_slots: Vec<SlotNumber> = records\n        .iter()\n        .filter(|record| {\n"
             + f"            {P}step({row('RecordIntersectionFilter')});\n",
             None))
    HEAD_MAKE_ROOM = "\nfn make_room_for_record_on_device(\n"
    e.extend(span("s27_filter_timing", A, HEAD_MAKE_ROOM, FN_END,
                  "    let intersecting_record_slots: Vec<SlotNumber> = records\n",
                  "    let placement_slot_was_reclaimed = reclaimed.contains(&(device, placement.slot));\n",
                  "RecordIntersectionFilter", "    ",
                  counts_end=f"    {P}delta(\"intersecting_records\", intersecting_record_slots.len());\n"))
    add(Edit("s27_find_steps", A, HEAD_MAKE_ROOM, FN_END,
             "                .find(|record| record.device == device && record.slot == record_slot)\n", "replace",
             "                .find(|record| {\n"
             + f"                    {P}step_to_the_hit({row('RecordIntersectionFilter')});\n"
             + "                    record.device == device && record.slot == record_slot\n                })\n", None))
    # 修订 4：find 之前读这一刻的记录数（走到命中为止那一部分的上界）
    e.extend(span("s27_find_timing", A, HEAD_MAKE_ROOM, FN_END,
                  "            let existing = records\n",
                  "            assert!(existing.is_released, \"回收过的落点的记录该是已释放\");\n",
                  "RecordIntersectionFilter", "            ",
                  counts_begin=f"            {P}scale_to_the_hit({row('RecordIntersectionFilter')}, records.len());\n"))
    retain_line = "            records.retain(|record| !(record.device == device && record.slot == record_slot));\n"
    # 修订 4：retain 走完这一刻的记录（全表扫那一部分），retain 之前读
    add(Edit("s27_retain", A, HEAD_MAKE_ROOM, FN_END, retain_line, "replace",
             f"            {P}scale({row('RecordIntersectionFilter')}, records.len());\n"
             "            records.retain(|record| {\n"
             + f"                {P}step({row('RecordIntersectionFilter')});\n"
             + "                !(record.device == device && record.slot == record_slot)\n            });\n",
             "            {\n"
             + f"                let _e203_timer = {P}timer({row('RecordIntersectionFilter')});\n"
             + "    " + retain_line.replace("            records", "            records", 1) + "            }\n"))
    GUARD_SCOPE["s27_retain"] = "block"
    # ---- transaction.rs 里的方法与函数
    add(Edit("s2_inode_leaf_container_copy", T,
             "    pub fn inode_leaf_container_contents(&self) -> Vec<InodeLeafContainer> {\n", METHOD_END,
             "        self.inode_leaf_containers\n            .iter()\n            .map(|container| container.contents.clone())\n            .collect()\n",
             "before",
             counted_call("InodeLeafContainerCopy")
             + f"        {P}steps({row('InodeLeafContainerCopy')}, self.inode_leaf_containers.iter().map(|container| container.contents.records.len()).sum::<usize>());\n",
             guard("InodeLeafContainerCopy")))
    find_f = "        self.files_other_than_the_first\n            .iter()\n            .find(|file| file.inode == inode)\n"
    add(Edit("s6_file_find", T, "    pub fn file_other_than_the_first(\n", METHOD_END, find_f, "replace",
             counted_call("FileOtherThanTheFirstLinearFind")
             # 修订 4 的上界：find 之前读这一版这一刻的别的文件数
             + f"        {P}scale_to_the_hit({row('FileOtherThanTheFirstLinearFind')}, self.files_other_than_the_first.len());\n"
             + "        self.files_other_than_the_first\n            .iter()\n            .find(|file| {\n"
             + f"                {P}step_to_the_hit({row('FileOtherThanTheFirstLinearFind')});\n"
             + "                file.inode == inode\n            })\n",
             guard("FileOtherThanTheFirstLinearFind") + find_f))
    add(Edit("s7_inodes_with_an_entry", T,
             "    pub fn inodes_with_an_extent_upper_leaf_entry(&self) -> BTreeSet<u64> {\n", METHOD_END,
             "        let first_file_has_an_entry = !self.data_pointers.is_empty()\n", "before",
             counted_call("InodesWithAnExtentEntrySet")
             + f"        {P}steps({row('InodesWithAnExtentEntrySet')}, 1 + self.files_other_than_the_first.len());\n",
             guard("InodesWithAnExtentEntrySet")))
    unit_find = "        self.units\n            .iter()\n            .find(|unit| unit.identity == identity)\n"
    add(Edit("s12_root_header_find", T, "    pub fn unit(&self, identity: TransactionUnit) -> &PublishedUnit {\n", METHOD_END,
             unit_find, "replace",
             counted_call("RootHeaderLinearFind")
             # 修订 4 的上界：find 之前读这一版这一刻的单元数
             + f"        {P}scale_to_the_hit({row('RootHeaderLinearFind')}, self.units.len());\n"
             + "        self.units\n            .iter()\n            .find(|unit| {\n"
             + f"                {P}step_to_the_hit({row('RootHeaderLinearFind')});\n"
             + "                unit.identity == identity\n            })\n",
             guard("RootHeaderLinearFind") + unit_find))
    sort_w = "        steps.sort_by_key(|entry| entry.generation);\n"
    add(Edit("s18_watermark_entry", T, "    pub fn inode_number_watermark_steps(&self) -> Vec<AccountingEntry> {\n", METHOD_END,
             "        let version_txg = self.root.checkpoint_txg;\n", "before",
             counted_call("InodeNumberWatermarkSteps")
             + f"        {P}steps({row('InodeNumberWatermarkSteps')}, 2 * self.accounting_entries.len());\n",
             guard("InodeNumberWatermarkSteps")))
    add(Edit("s18_watermark_sort", T, "    pub fn inode_number_watermark_steps(&self) -> Vec<AccountingEntry> {\n", METHOD_END,
             sort_w, "replace",
             "        steps.sort_by(|left, right| {\n"
             + f"            {P}step({row('InodeNumberWatermarkSteps')});\n"
             + "            left.generation.cmp(&right.generation)\n        });\n", None))
    add(Edit("s5_files_plan", T, "\npub fn plan_the_files_other_than_the_first_after_this_publish(\n", FN_END,
             "    let changed_inode = change.inode();\n", "after",
             counted_call("FilesOtherThanTheFirstPlan", "    ")
             + f"    {P}steps({row('FilesOtherThanTheFirstPlan')}, previous.map_or(0, |version| version.files_other_than_the_first.len()));\n"
             + f"    {P}steps({row('FilesOtherThanTheFirstLowerNodeCopy')}, previous.map_or(0, |version| version.files_other_than_the_first.iter().map(|file| file.extent_lower_nodes.len()).sum::<usize>()));\n"
             + f"    {P}delta(\"files_other_than_the_first_before\", previous.map_or(0, |version| version.files_other_than_the_first.len()));\n"
             + f"    {P}delta(\"touched_files\", usize::from(changed_inode.is_some()));\n"
             + f"    {P}delta(\"lower_nodes_of_touched_files\", changed_inode.map_or(0, |touched| previous.map_or(0, |version| version.files_other_than_the_first.iter().find(|file| file.inode == touched).map_or(0, |file| file.extent_lower_nodes.len()))));\n",
             None))
    e.extend(span("s5_timing", T, "\npub fn plan_the_files_other_than_the_first_after_this_publish(\n", FN_END,
                  "    let mut files: Vec<FileOtherThanTheFirstPlan> = previous\n",
                  "    let previous_file_of = |inode: InodeNumber| {\n", "FilesOtherThanTheFirstPlan", "    "))
    add(Edit("s11_roles_rewritten", T, "\npub fn extent_tree_roles_rewritten(\n", FN_END,
             "    let upper_root_level = extent_tree.upper_root_level();\n", "before",
             counted_call("ExtentRolesRewrittenScan", "    ")
             + f"    {P}steps({row('ExtentRolesRewrittenScan')}, extent_tree.lower_nodes.len() + extent_tree.upper_nodes.len() + files_other_than_the_first.files.iter().map(|file| file.lower_nodes.len()).sum::<usize>());\n",
             guard("ExtentRolesRewrittenScan", "    ")))
    add(Edit("s31_accounting_entries", T, "\nfn accounting_entries_of_this_publish(\n", FN_END,
             "    entries.extend_from_slice(inode_number_watermark_steps_carried_from_the_previous_version);\n", "before",
             counted_call("AccountingEntriesOfThisPublish", "    ")
             + f"    {P}steps({row('AccountingEntriesOfThisPublish')}, entries.len());\n"
             + f"    {P}delta(\"accounting_rows_of_the_generation\", entries.len());\n", None))
    add(Edit("s20_attribution", T, "    fn plan_the_multi_level_trees(\n", METHOD_END,
             "        let txg = self.txg;\n        let instance = self.instance;\n        let empty_shape = CodeTwoTreeShape::default();\n",
             "before",
             f"        let _e203_attribution = {P}attribute({row('MultiLevelTreePlanning')});\n",
             f"        let _e203_attribution = {P}attribute({row('MultiLevelTreePlanning')});\n"))
    # ---- admission.rs、code_two_tree.rs、allocation_record_tree.rs
    ADM = CORE + "admission.rs"
    CODE2 = CORE + "code_two_tree.rs"
    ART = CORE + "allocation_record_tree.rs"
    attr13 = f"    let _e203_attribution = {P}attribute({row('CheckpointCostAccountingTreePlan')});\n"
    add(Edit("s13_entry", ADM, "\npub fn accounting_tree_nodes_after_the_publish_built_on(\n", FN_END,
             "    let keys_after_that_publish: BTreeSet<CodeTwoTreeKey> = rows_written_by_that_publish\n", "before",
             attr13 + counted_call("CheckpointCostAccountingTreePlan", "    ")
             + f"    {P}steps({row('CheckpointCostAccountingTreePlan')}, version_to_build_on.accounting_entries.len());\n"
             + f"    {P}delta(\"accounting_rows_written_by_the_built_on_publish\", version_to_build_on.accounting_entries.iter().filter(|entry| entry.generation == version_txg).count());\n",
             attr13 + guard("CheckpointCostAccountingTreePlan", "    ")))
    add(Edit("s13_keys", ADM, "\npub fn accounting_tree_nodes_after_the_publish_built_on(\n", FN_END,
             "    assert!(\n        keys_after_that_publish.len() <= version_to_build_on.accounting_entries.len() + 1,\n", "before",
             f"    {P}steps({row('CheckpointCostAccountingTreePlan')}, keys_after_that_publish.len());\n"
             f"    {P}delta(\"accounting_keys_after_that_publish\", keys_after_that_publish.len());\n", None))
    add(Edit("s33_units_to_land", ADM, "\npub fn units_of_a_publish_to_land(\n", FN_END,
             "    let data_units = ordinary_allocation_roles\n", "before",
             counted_call("UnitsOfAPublishToLand", "    ")
             + f"    {P}steps({row('UnitsOfAPublishToLand')}, 2 * ordinary_allocation_roles.len());\n",
             guard("UnitsOfAPublishToLand", "    ")))
    add(Edit("code_two_plan_timer", CODE2, "\npub fn plan_the_tree_after_this_publish(\n", FN_END,
             "    let previous_keys: BTreeSet<CodeTwoTreeKey> =\n", "before",
             None, f"    let _e203_timer = {P}timer_attributed();\n"))
    add(Edit("code_two_plan_steps", CODE2, "\npub fn plan_the_tree_after_this_publish(\n", FN_END,
             "    Ok(CodeTwoTreePlan {\n        shape: CodeTwoTreeShape { nodes },\n", "before",
             f"    {P}call_attributed();\n"
             f"    {P}steps_attributed(previous_keys.len() + keys_after_this_publish.len() + nodes.len() + origins.len() + previous.nodes().len());\n"
             f"    {P}delta_if_attributed({row('MultiLevelTreePlanning')}, \"code_two_keys_changed\", previous_keys.symmetric_difference(&keys_after_this_publish).count());\n"
             f"    {P}delta(\"code_two_plan_keys_after\", keys_after_this_publish.len());\n"
             f"    {P}delta(\"code_two_plan_nodes_after\", nodes.len());\n", None))
    # deadlist 树这一版一把 key 都不剩时走零根那一支，不进上面的规划函数：这一支也是一棵码 2 树的一次规划，
    # 照记一次调用、走过的上一版节点（换下的每一个），与进出的 key 数（上一版的 key 全出、没有进的），每轮三棵树各一条。
    zero_root_keys_changed = (f"    {P}delta_if_attributed({row('MultiLevelTreePlanning')}, \"code_two_keys_changed\", "
                              "previous.keys_in_order().len());\n")
    add(Edit("code_two_zero_root_plan_steps", CODE2, HEAD_ZERO_ROOT, FN_END,
             "    if !keys_after_this_publish.is_empty() {\n"
             "        return plan_the_tree_after_this_publish(previous, keys_after_this_publish, capacity);\n    }\n",
             "after",
             f"    {P}call_attributed();\n"
             f"    {P}steps_attributed(previous.nodes().len());\n"
             + zero_root_keys_changed,
             None))
    add(Edit("s16_allocation_plan", ART, "\npub fn plan_the_tree_after_this_publish(\n", FN_END,
             "    let nodes = nodes_after\n        .iter()\n        .map(|node| {\n", "before",
             counted_call("AllocationRecordTreePlan", "    ")
             # 修订 4：这一次的节点与上一版的节点各走一遍（全表扫，逐格相等），调用那一刻读两张表长
             + f"    {P}scale({row('AllocationRecordTreePlan')}, nodes_after.len() + previous_nodes.len());\n"
             + f"    {P}steps({row('AllocationRecordTreePlan')}, nodes_after.len() + previous_nodes.len());\n"
             + f"    {P}delta(\"allocation_plan_nodes_after\", nodes_after.len());\n",
             guard("AllocationRecordTreePlan", "    ")))
    add(Edit("s30_records_of_each_leaf", ART, "\npub fn records_of_each_leaf(\n", FN_END,
             "    let mut by_leaf: BTreeMap<AllocationRecordTreeNode, Vec<AllocationRecord>> = BTreeMap::new();\n", "before",
             counted_call("RecordsOfEachLeaf", "    ")
             # 修订 4：每条记录放进它的叶（全表扫，逐格相等），调用那一刻读记录数
             + f"    {P}scale({row('RecordsOfEachLeaf')}, records.len());\n"
             + f"    {P}steps({row('RecordsOfEachLeaf')}, records.len());\n",
             guard("RecordsOfEachLeaf", "    ")))
    add_the_rest(e)
    for ident in BODY_IDENTS:
        GUARD_SCOPE[ident] = "body"
    for ident in ("s1_caller_copy", "s24_rehearsal_copy"):
        GUARD_SCOPE[ident] = "block"
    return e


# 函数体就是那一步的计时守卫（体里没有别的步）；逐个读过源码判的，判完的理由见登记第十二节修订 1。
BODY_IDENTS = (
    "s2_inode_leaf_container_copy", "s6_file_find", "s7_inodes_with_an_entry", "s12_root_header_find",
    "s18_watermark_entry", "s11_roles_rewritten", "s13_entry", "s33_units_to_land", "code_two_plan_timer",
    "s16_allocation_plan", "s30_records_of_each_leaf", "s4_inode_record_find", "s8_timer",
    "s21_container_tree_mapped", "s32_instance_table_chain", "s30_changed_timer", "index_build_steps",
    "s31_timer",
)


def add_the_rest(e):
    """第二批站点：第一段没落的 12 行与半落的几处，加上 B8 的输入读数。"""
    add = e.append
    HEAD_ORD = "    fn ordinary_allocation_part(\n"
    HEAD_PLAN_TREES = "    fn plan_the_multi_level_trees(\n"
    one = "usize::from"
    # ---- s3 inode 叶写者拷、s9 墓碑容器拷、s10 退役容器拷（调用方记；写者在 Verus 块里不能插）
    e.extend(span("s3_inode_leaf_writer_copy", T, HEAD_ORD, METHOD_END,
                  "        let inode_tree = match deletion_and_lineage.step {\n",
                  "        let inode_record_counts_before = previous\n", "InodeLeafWriterCopy", "        ",
                  counts_begin=counted_call("InodeLeafWriterCopy")
                  + f"        {P}steps({row('InodeLeafWriterCopy')}, containers_before.iter().map(|container| container.records.len()).sum::<usize>());\n"))
    e.extend(span("s9_tombstone_container_copy", T, HEAD_ORD, METHOD_END,
                  "        let tombstone_containers_before: Vec<TombstoneContainer> = previous\n",
                  "        let zero_root_shape = ContainerTreeShape::zero_root();\n", "TombstoneContainerCopy", "        ",
                  counts_begin=counted_call("TombstoneContainerCopy"),
                  counts_end=f"        {P}steps({row('TombstoneContainerCopy')}, tombstone_containers_before.iter().map(|container| container.records.len()).sum::<usize>());\n"))
    e.extend(span("s10_retirement_container_copy", T, HEAD_ORD, METHOD_END,
                  "        let retirement_containers_before: Vec<RetirementContainer> = previous\n",
                  "        let retirement_shape_before: &ContainerTreeShape = previous\n", "RetirementContainerCopy", "        ",
                  counts_begin=counted_call("RetirementContainerCopy"),
                  counts_end=f"        {P}steps({row('RetirementContainerCopy')}, retirement_containers_before.iter().map(|container| container.records.len()).sum::<usize>());\n"))
    e.extend(span("s17_lineage_container_copy", T, "    fn lineage_tree_after_this_publish(\n", METHOD_END,
                  "        let containers_before: Vec<LineageContainer> = previous\n",
                  "        let zero_root_shape = ContainerTreeShape::zero_root();\n", "LineageContainerCopy", "        ",
                  counts_begin=counted_call("LineageContainerCopy"),
                  counts_end=f"        {P}steps({row('LineageContainerCopy')}, containers_before.iter().map(|container| container.records.len()).sum::<usize>());\n"))
    # ---- s4 inode 记录线性找
    inode_find = ("        self.inode_leaf_containers\n            .iter()\n            .flat_map(|container| container.contents.records.iter())\n"
                  "            .find(|record| record.inode == inode.0)\n")
    add(Edit("s4_inode_record_find", T, "    pub fn inode_record_of(&self, inode: InodeNumber) -> Option<InodeRecord> {\n", METHOD_END,
             inode_find, "replace",
             counted_call("InodeRecordLinearFind")
             + "        self.inode_leaf_containers\n            .iter()\n            .flat_map(|container| container.contents.records.iter())\n"
             + "            .find(|record| {\n" + f"                {P}step({row('InodeRecordLinearFind')});\n"
             + "                record.inode == inode.0\n            })\n",
             guard("InodeRecordLinearFind") + inode_find))
    # ---- s8 extent 上段统一计划（extent_tree.rs）
    EXT = CORE + "extent_tree.rs"
    HEAD_S8 = "\npub fn plan_the_extent_upper_segment(\n"
    add(Edit("s8_timer", EXT, HEAD_S8, FN_END,
             "    let (_, positions_after_this_publish) = canonical_upper_segment_of_the_inodes_with_an_entry(\n", "before",
             counted_call("ExtentUpperSegmentPlan", "    "), guard("ExtentUpperSegmentPlan", "    ")))
    add(Edit("s8_path_steps", EXT, HEAD_S8, FN_END,
             "    let covers_a_changed_inode = |position: &ExtentUpperNodePosition| {\n", "before",
             f"    {P}steps({row('ExtentUpperSegmentPlan')}, inodes_with_an_entry_after_this_publish.len() * (1 + positions_after_this_publish.iter().map(|position| usize::from(position.level)).max().unwrap_or(0)));\n",
             None))
    # 修订 4：路径那一部分是全表扫（每个有条目的号走一条到根的路），调用那一刻读有条目的号数与新形状的最高层，与记步数那一句分开读
    add(Edit("s8_path_table", EXT, HEAD_S8, FN_END,
             "    let covers_a_changed_inode = |position: &ExtentUpperNodePosition| {\n", "before",
             f"    {P}scale({row('ExtentUpperSegmentPlan')}, inodes_with_an_entry_after_this_publish.len() * (1 + positions_after_this_publish.iter().map(|position| usize::from(position.level)).max().unwrap_or(0)));\n",
             None))
    contains_line = "                if covers_a_changed_inode(position) || !previous_upper_nodes.contains(position) {\n"
    # 修订 4 的上界：contains 只在没被短路时调，每调一次之前读上一版上段的节点数；走过的步记在走到命中为止那一部分（contains_counted）
    add(Edit("s8_contains_steps", EXT, HEAD_S8, FN_END, contains_line, "replace",
             f"                if covers_a_changed_inode(position) || !{{ {P}scale_to_the_hit({row('ExtentUpperSegmentPlan')}, previous_upper_nodes.len());"
             f" {P}contains_counted({row('ExtentUpperSegmentPlan')}, previous_upper_nodes, position) }} {{\n",
             None))
    # 修订 4：换下的那一遍 filter 走完上一版上段的每个节点（全表扫），filter 之前读
    add(Edit("s8_replaced_filter_table", EXT, HEAD_S8, FN_END,
             "    let replaced_previous_positions = previous_upper_nodes\n", "before",
             f"    {P}scale({row('ExtentUpperSegmentPlan')}, previous_upper_nodes.len());\n", None))
    replaced_filter = ("        .filter(|position| {\n"
                       "            !positions_after_this_publish.contains(position) || covers_a_changed_inode(position)\n")
    add(Edit("s8_replaced_filter_steps", EXT, HEAD_S8, FN_END, replaced_filter, "replace",
             "        .filter(|position| {\n" + f"            {P}step({row('ExtentUpperSegmentPlan')});\n"
             + "            !positions_after_this_publish.contains(position) || covers_a_changed_inode(position)\n", None))
    add(Edit("s8_delta", EXT, HEAD_S8, FN_END, "    ExtentUpperSegmentPlan {\n", "before",
             f"    {P}delta(\"extent_upper_nodes_rewritten\", nodes.iter().filter(|(_, origin)| *origin == ExtentTreeNodeOrigin::RewrittenThisPublish).count());\n",
             None))
    # ---- s16 固定点每轮的 intersection
    add(Edit("s16_intersection_steps", T, HEAD_SETTLE, FN_END,
             "            rewritten.intersection(&nodes_after).copied().collect();\n", "after",
             f"        {P}steps({row('AllocationRecordTreePlan')}, rewritten.len() + nodes_after.len());\n", None))
    # 修订 4：intersection 那两张表在这一轮开头的长度（全表扫，逐格相等），intersection 之前读
    e.extend(span("s16_intersection_timing", T, HEAD_SETTLE, FN_END,
                  "        let rewritten_among_the_nodes_after: BTreeSet<AllocationRecordTreeNode> =\n",
                  "        let allocation_plan = crate::allocation_record_tree::plan_the_tree_after_this_publish(\n",
                  "AllocationRecordTreePlan", "        ",
                  counts_begin=f"        {P}scale({row('AllocationRecordTreePlan')}, rewritten.len() + nodes_after.len());\n"))
    e.extend(span("s15_timing", T, HEAD_SETTLE, FN_END,
                  "    let (previous_nodes, previous_records): (\n",
                  "    for _round in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {\n",
                  "AllocationRecordTreeNodeSet", "    "))
    # ---- s30 每比一片叶，与 B8 的 dirty、λ_ar
    ART = CORE + "allocation_record_tree.rs"
    HEAD_CHANGED = "\npub fn nodes_whose_contents_changed(\n"
    add(Edit("s30_changed_timer", ART, HEAD_CHANGED, FN_END,
             "    let before = records_of_each_leaf(geometry, previous_records);\n", "before",
             None, guard("RecordsOfEachLeaf", "    ")))
    add(Edit("s30_changed_steps", ART, HEAD_CHANGED, FN_END,
             "        before.keys().chain(after.keys()).copied().collect();\n", "after",
             # 修订 4：两边叶的并每片比一次（全表扫，逐格相等），比之前读并集的片数
             f"    {P}scale({row('RecordsOfEachLeaf')}, leaves.len());\n"
             f"    {P}steps({row('RecordsOfEachLeaf')}, leaves.len());\n"
             "    {\n"
             "        let no_records: Vec<AllocationRecord> = Vec::new();\n"
             "        let mut dirty_records = 0_usize;\n"
             "        for leaf in &leaves {\n"
             "            let leaf_before = before.get(leaf).unwrap_or(&no_records);\n"
             "            let leaf_after = after.get(leaf).unwrap_or(&no_records);\n"
             "            if leaf_before != leaf_after {\n"
             "                dirty_records += leaf_before.iter().filter(|record| !leaf_after.contains(record)).count()\n"
             "                    + leaf_after.iter().filter(|record| !leaf_before.contains(record)).count();\n"
             "            }\n"
             "        }\n"
             f"        {P}delta(\"dirty_records\", dirty_records);\n"
             f"        {P}delta(\"changed_leaves\", leaves.iter().filter(|leaf| before.get(*leaf) != after.get(*leaf)).count());\n"
             "    }\n", None))
    # ---- s19 deadlist 改动的索引
    HEAD_DEADLIST = "\nfn deadlist_changes_of_this_publish(\n"
    attribute_19 = f"    let _e203_attribution = {P}attribute({row('DeadlistChangesIndexes')});\n"
    add(Edit("s19_attribution", T, HEAD_DEADLIST, FN_END,
             "    let _inside_the_snapshot_judgement = lookup_work::entering(LookupSite::SnapshotJudgement);\n", "before",
             attribute_19 + counted_call("DeadlistChangesIndexes", "    "), attribute_19))
    e.extend(span("s19_entries_after", T, HEAD_DEADLIST, FN_END,
                  "    let mut entries_after: BTreeMap<CodeTwoTreeKey, DeadlistEntry> = previous_version\n",
                  "    for (entry, _) in &entries_released_by_the_destruction {\n", "DeadlistChangesIndexes", "    ",
                  counts_end=f"    {P}steps({row('DeadlistChangesIndexes')}, previous_version.deadlist_entries.len() + previous_version.mapping_entries_kept_for_the_snapshots.len());\n"))
    add(Edit("s19_delta", T, HEAD_DEADLIST, FN_END, "    Ok(DeadlistChangesOfThisPublish {\n", "before",
             f"    {P}delta(\"deadlist_entries_changed\", user_visible_roles_kept_for_the_snapshots.len() + entries_released_by_the_destruction.len());\n",
             None))
    # ---- s21 三棵容器树按角色出映射单元
    HEAD_CONTAINER_MAPPED = "\nfn mapped_units_of_a_container_tree<Record>(\n"
    add(Edit("s21_container_tree_mapped", T, HEAD_CONTAINER_MAPPED, FN_END,
             "    let mut mapped_units = Vec::new();\n    let mut birth_sequence = 0;\n", "before",
             counted_call("MappedUnitsOfContainerTrees", "    "), guard("MappedUnitsOfContainerTrees", "    ")))
    add(Edit("s21_container_tree_steps", T, HEAD_CONTAINER_MAPPED, FN_END, "    mapped_units\n}\n", "before",
             f"    {P}steps({row('MappedUnitsOfContainerTrees')}, mapped_units.len());\n"
             f"    {P}delta(\"container_units_rewritten\", tree.containers.len().saturating_sub(first_rewritten_container.unwrap_or(tree.containers.len())) + tree.node_origins().iter().filter(|(_, origin)| matches!(origin, ContainerTreeNodeOrigin::RewrittenThisPublish)).count());\n",
             None))
    # ---- s22 搬迁的尾部过滤
    HEAD_MOVED = "\nfn plan_rewriting_the_paths_to_the_moved_entries(\n"
    e.extend(span("s22_moved_tail", T, HEAD_MOVED, FN_END,
                  "    plan.replaced_previous_nodes = previous_shape\n", "    plan\n}\n", "MovedEntriesPathTailFilter", "    ",
                  counts_begin=counted_call("MovedEntriesPathTailFilter", "    ")
                  + f"    {P}steps({row('MovedEntriesPathTailFilter')}, previous_shape.nodes().len());\n"))
    # ---- s23 释放判定的两份全版索引
    attribute_23 = f"    let _e203_attribution = {P}attribute({row('ReleaseJudgementVersionIndexes')});\n"
    add(Edit("s23_attribution", T,
             "\npub fn placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots(\n", FN_END,
             "    let _inside_the_releases =\n        lookup_work::entering(LookupSite::MappingEntriesOfTheReleaseJudgement);\n", "before",
             attribute_23 + counted_call("ReleaseJudgementVersionIndexes", "    "), attribute_23))
    # ---- s32 实例表链
    add(Edit("s32_instance_table_chain", T, "\npub fn instance_table_chain_to_release(\n", FN_END,
             "    replaced_chain\n        .iter()\n        .enumerate()\n", "before",
             counted_call("InstanceTableChain", "    ")
             + f"    {P}steps({row('InstanceTableChain')}, replaced_chain.len());\n",
             guard("InstanceTableChain", "    ")))
    # ---- 多层码 2 树规划里除规划函数本身之外的三处收集，与 s36 的整表
    accounting_keys = "        let accounting_keys: BTreeSet<CodeTwoTreeKey> = accounting_entries_of_this_publish(\n"
    e.extend(span("s20_accounting_keys", T, HEAD_PLAN_TREES, METHOD_END, accounting_keys,
                  "        let accounting_tree = plan_the_tree_after_this_publish(\n", "MultiLevelTreePlanning", "        ",
                  counts_end=f"        {P}steps({row('MultiLevelTreePlanning')}, accounting_keys.len());\n"))
    e.extend(span("s20_deadlist_keys", T, HEAD_PLAN_TREES, METHOD_END,
                  "        let deadlist_keys: BTreeSet<CodeTwoTreeKey> = deadlist\n",
                  "        let deadlist_tree = plan_the_tree_after_this_publish_down_to_a_zero_root(\n",
                  "MultiLevelTreePlanning", "        ",
                  counts_end=f"        {P}steps({row('MultiLevelTreePlanning')}, deadlist_keys.len());\n"))
    e.extend(span("s36_mapped_units_table", T, HEAD_PLAN_TREES, METHOD_END,
                  "        let mapped_units_of_the_previous_version =\n            previous.map(TransactionOutput::mapped_units_by_role);\n",
                  "        let mapping_keys: BTreeSet<CodeTwoTreeKey> = mapped_units\n", "MappedUnitsWholeTable", "        ",
                  counts_begin=counted_call("MappedUnitsWholeTable"),
                  counts_end=f"        {P}steps({row('MappedUnitsWholeTable')}, mapped_units.len());\n",
                  attribution=True))
    e.extend(span("s20_mapping_keys", T, HEAD_PLAN_TREES, METHOD_END,
                  "        let mapping_keys: BTreeSet<CodeTwoTreeKey> = mapped_units\n",
                  "        assert_eq!(\n            mapping_keys.len(),\n", "MultiLevelTreePlanning", "        ",
                  counts_end=f"        {P}steps({row('MultiLevelTreePlanning')}, mapping_keys.len());\n"))
    add(Edit("s22_moved_entries_delta", T, HEAD_PLAN_TREES, METHOD_END,
             "        let central_mapping_tree = plan_rewriting_the_paths_to_the_moved_entries(\n", "before",
             f"        {P}delta(\"moved_entries\", mapping_keys_moved_by_this_publish.len());\n", None))
    add(Edit("s31_timer", T, "\nfn accounting_entries_of_this_publish(\n", FN_END,
             "    let mut entries: Vec<AccountingEntry> = PoolWideAccountingRow::ALL\n", "before",
             None, guard("AccountingEntriesOfThisPublish", "    ")))
    # ---- 按身份建的索引（所有行共用）：建一次记 table.len() 步，查一次记一层数
    add(Edit("index_build_steps", T, "    pub fn of_the_table(\n", METHOD_END,
             "        let mut identities_with_their_entries = Vec::with_capacity(table.len());\n", "before",
             f"        {P}call_attributed();\n        {P}steps_attributed(table.len());\n",
             f"        let _e203_timer = {P}timer_attributed();\n"))
    add(Edit("index_query_levels", T,
             "        pub(super) fn value_stored_for_identity(&self, identity: &Identity) -> Option<&Value> {\n", "\n        }\n",
             "            let index_entries = u64::try_from(self.sorted.len()).expect(\"索引项数装得进 u64\");\n", "after",
             f"            {P}query_attributed(index_entries);\n", None))


ALLOCATOR_HELPERS = '''
/// E203（准入之前扫类的步代价）只读出的帮助：一份整份拷贝要拷的集合元素个数，与这次挂载开过的聚簇段数。不进任何判定。
impl PoolAllocator {
    #[must_use]
    pub fn e203_whole_copy_elements(&self) -> usize {
        let per_device: usize = self.devices.iter().map(DeviceFreeMap::e203_whole_copy_elements).sum();
        per_device
            + self.records.len()
            + self.cluster_segments.len()
            + self.reclaimed.len()
            + self.root_ring.as_ref().map_or(0, RootRingOccupancy::e203_elements)
    }

    #[must_use]
    pub fn e203_cluster_segments_opened(&self) -> usize {
        self.cluster_segments.len()
    }
}

impl DeviceFreeMap {
    fn e203_whole_copy_elements(&self) -> usize {
        self.allocated.len()
            + self.isolated.len()
            + self.held_until_floor_takes_effect.len()
            + self.used_per_segment.len()
            + self.unreleased_per_segment.len()
            + self.isolated_per_segment.len()
            + self.held_per_segment.len()
            + self.unblocked_slot_pairs_per_segment.len()
            + self.segments_reclaimed_not_yet_discarded.len()
    }
}

impl RootRingOccupancy {
    fn e203_elements(&self) -> usize {
        self.occupants.len()
            + self
                .placements_named_by_unapplied_journal_records
                .values()
                .map(Vec::len)
                .sum::<usize>()
    }
}
'''
HARNESS_FEATURES_FROM = 'features = ["lookup-work-counters", "test-only-publish-thresholds"]'
HARNESS_FEATURES_TO = 'features = ["test-only-publish-thresholds"]'


class AnchorError(Exception):
    pass


def module_text(build):
    module = f"e203_scan_{build}"
    variants = "\n".join(f"        {camel}," for camel, _, _ in ROWS)
    listing = "\n".join(f"        Row::{camel}," for camel, _, _ in ROWS)
    names = "\n".join(f'                Row::{camel} => "{snake}",' for camel, snake, _ in ROWS)
    keep, drop = ("//C ", "//T ") if build == "counts" else ("//T ", "//C ")
    lines = []
    for line in MODULE_TEMPLATE.splitlines():
        if line.startswith(drop):
            continue
        lines.append(line[len(keep):] if line.startswith(keep) else line)
    text = "\n".join(lines) + "\n"
    return (text.replace("@MODULE@", module).replace("@ROW_VARIANTS@", variants)
            .replace("@ROW_COUNT@", str(len(ROWS))).replace("@ROW_LIST@", listing).replace("@ROW_NAMES@", names))


def body_span(text, edit):
    if text.count(edit.head) != 1:
        raise AnchorError(f"{edit.ident}: 函数头在 {edit.file} 里命中 {text.count(edit.head)} 次，不是一次：{edit.head.strip()}")
    start = text.index(edit.head)
    stop = text.index(edit.end, start + len(edit.head)) + len(edit.end)
    return start, stop


def locate(text, edit):
    start, stop = body_span(text, edit)
    hits = text[start:stop].count(edit.anchor)
    if hits != 1:
        raise AnchorError(f"{edit.ident}: 锚点在 {edit.head.strip()} 的函数体里命中 {hits} 次，不是一次：{edit.anchor.strip()[:100]}")
    return start + text[start:stop].index(edit.anchor)


def apply_one(text, edit, build):
    position = locate(text, edit)
    inserted = edit.counts if build == "counts" else edit.timing
    if inserted is None:
        return text
    end = position + len(edit.anchor)
    if edit.mode == "before":
        return text[:position] + inserted + text[position:]
    if edit.mode == "after":
        return text[:end] + inserted + text[end:]
    return text[:position] + inserted + text[end:]


def transform(files, build, edit_list):
    """files: {相对路径: 原文}。交回 {相对路径: 新文}；先把所有锚点核一遍，有一个不是恰好一次就一个字节都不改。"""
    by_file = {}
    for edit in edit_list:
        by_file.setdefault(edit.file, []).append(edit)
    for path, group in by_file.items():
        if path not in files:
            raise AnchorError(f"文件不在：{path}")
        for edit in group:
            locate(files[path], edit)
    result = dict(files)
    for path, group in by_file.items():
        text = files[path]
        for edit in group:
            text = apply_one(text, edit, build)
        result[path] = text
    result[T] = result[T] + module_text(build)
    result[A] = result[A] + ALLOCATOR_HELPERS
    if build == "timing":
        harness = "crates/singlefs-verification/Cargo.toml"
        if result[harness].count(HARNESS_FEATURES_FROM) != 1:
            raise AnchorError(f"{harness} 里 lookup-work-counters 的特性行命中 {result[harness].count(HARNESS_FEATURES_FROM)} 次，不是一次")
        result[harness] = result[harness].replace(HARNESS_FEATURES_FROM, HARNESS_FEATURES_TO)
    return result


def unified(files, changed):
    out = []
    for path in sorted(changed):
        if files[path] != changed[path]:
            out.extend(difflib.unified_diff(files[path].splitlines(keepends=True), changed[path].splitlines(keepends=True),
                                            f"a/{path}", f"b/{path}"))
    return "".join(out)


def check_builds_are_not_mixed(build, diff_text):
    """作废条款第 7 条：计时构建的 diff 里不许有计数模块，计数构建里不许有计时。"""
    counting_calls = [f"{P}{name}(" for name in ("call", "step", "steps", "scale", "delta", "call_attributed", "steps_attributed", "round_begin",
                                                 "contains_counted", "query_attributed", "delta_if_attributed",
                                                 "scale_to_the_hit", "steps_to_the_hit", "step_to_the_hit")]
    if build == "timing" and ("e203_scan_counts" in diff_text or any(call in diff_text for call in counting_calls)):
        return "计时构建的 diff 里出现了计数"
    if build == "counts" and ("e203_scan_timing" in diff_text or "Instant" in diff_text or f"{P}timer(" in diff_text
                              or f"{P}timer_attributed(" in diff_text):
        return "计数构建的 diff 里出现了计时"
    return None


def read_tree(root, needed):
    files = {}
    for path in needed:
        file = pathlib.Path(root) / path
        if not file.is_file():
            print(f"不在：{file}", file=sys.stderr)
            sys.exit(2)
        files[path] = file.read_text(encoding="utf-8")
    return files


def needed_paths():
    paths = {edit.file for edit in edits()}
    paths.add("crates/singlefs-verification/Cargo.toml")
    return sorted(paths)


# 副本要编得起来必须有的：工作区清单与锁文件、所有包、cargo 配置、harness 包编译时读的每台默认线程数登记（`include_str!`）、
# 仓根 `Cargo.toml` 的 `exclude` 里点名而 验证包用 path 依赖带进来的 `prune-server`（缺了 `cargo metadata --offline` 解析不出）。
COPIED_DIRECTORIES = ("crates", ".cargo", "prune-server")
COPIED_FILES = ("Cargo.toml", "Cargo.lock", ".claude/heavy-thread-default")


class CopyError(Exception):
    pass


def manifest_paths_not_in(root):
    """root 底下每份清单里的 path 依赖与工作区成员：指到的目录（或 .rs 文件）不在 root 里的，逐条列出。"""
    import re
    root = pathlib.Path(root)
    problems = []
    manifests = [root / "Cargo.toml"]
    for name in ("crates", "prune-server"):
        if (root / name).is_dir():
            manifests.extend(sorted(path for path in (root / name).rglob("Cargo.toml") if "target" not in path.parts))
    for manifest in manifests:
        if not manifest.is_file():
            problems.append(f"{manifest.relative_to(root)} 不在")
            continue
        text = manifest.read_text(encoding="utf-8")
        for target in re.findall(r'path\s*=\s*"([^"]+)"', text):
            resolved = (manifest.parent / target).resolve()
            if not resolved.exists():
                problems.append(f"{manifest.relative_to(root)} 的 path 依赖 {target} 指到副本里不存在的位置")
        if manifest == root / "Cargo.toml":
            members = re.search(r"members\s*=\s*\[(.*?)\]", text, re.S)
            for member in re.findall(r'"([^"]+)"', members.group(1) if members else ""):
                if not (root / member / "Cargo.toml").is_file():
                    problems.append(f"工作区成员 {member} 在副本里没有 Cargo.toml")
    return problems


COPY_BEGUN = ".e203-copy-begun"
COPY_DONE = ".e203-copy-done"
COPY_RESUME_OFF = "copy-resume-off"


def write_atomically(path, text):
    """先写同目录的 .partial，再换名：被杀时目标名下要么是旧的整份、要么没有，没有半份。"""
    path = pathlib.Path(path)
    temporary = path.with_name(path.name + ".partial")
    temporary.write_text(text, encoding="utf-8")
    os.replace(temporary, path)


def check_source_is_complete(source):
    missing = [name for name in COPIED_DIRECTORIES + COPIED_FILES if not (pathlib.Path(source) / name).exists()]
    if missing:
        raise CopyError(f"仓根里缺这几份，副本编不起来：{missing}")


def source_files_to_copy(source):
    """要拷的每一份文件的路径（相对 source，按次序），与 copytree 的 ignore 同口径：名叫 target 的目录与文件不要。"""
    source = pathlib.Path(source)
    paths = []
    for name in COPIED_DIRECTORIES:
        for directory, subdirectories, names in os.walk(source / name, followlinks=True):
            subdirectories[:] = sorted(entry for entry in subdirectories if entry != "target")
            paths.extend(pathlib.Path(directory, entry).relative_to(source) for entry in sorted(names) if entry != "target")
    paths.extend(pathlib.Path(name) for name in COPIED_FILES)
    return paths


def copy_key(source, kind, diff_text):
    """副本的输入键：构建种类、本脚本全文、改动的 diff 全文、要拷的每份文件的路径与内容。键相同，上一次拷完的副本就是这一次要的副本。"""
    digest = hashlib.sha256()
    for part in (kind.encode("utf-8"), pathlib.Path(__file__).read_bytes(), diff_text.encode("utf-8")):
        digest.update(hashlib.sha256(part).digest())
    try:
        for relative in source_files_to_copy(source):
            digest.update(str(relative).encode("utf-8") + b"\0")
            digest.update(hashlib.sha256((pathlib.Path(source) / relative).read_bytes()).digest())
    except OSError as error:
        raise CopyError(f"读不了要拷的文件：{error}") from error
    return digest.hexdigest()


def prepare_copy(source, into, key):
    """决定这一次要不要拷。交回 "reused"（上一次拷完、键相同的副本，原样用）或 "fresh"（目录是空的，或上一次没拷完、键不同的副本已清掉，可以拷）。
    非空而没有开拷标记的目录不是本脚本拷的：抛 CopyError，什么也不清。"""
    source, into = pathlib.Path(source), pathlib.Path(into)
    check_source_is_complete(source)
    into.mkdir(parents=True, exist_ok=True)
    entries = [entry.name for entry in into.iterdir()]
    if not entries:
        return "fresh"
    resume_off = os.environ.get("E203_RUN_BREAK") == COPY_RESUME_OFF
    done = into / COPY_DONE
    if not resume_off and done.is_file() and done.read_text(encoding="utf-8") == key + "\n":
        return "reused"
    if resume_off or not (COPY_BEGUN in entries or COPY_BEGUN + ".partial" in entries):
        raise CopyError(f"工作目录不空：{into}（没有开拷标记 {COPY_BEGUN}，不是本脚本拷的，不清）")
    # 半拷或输入变了：先拿掉完成标记（清到一半被杀也不会被当成拷完），再清其余，开拷标记留到最后由重拷改写
    if done.exists():
        done.unlink()
    for entry in sorted(into.iterdir()):
        if entry.name == COPY_BEGUN:
            continue
        if entry.is_dir() and not entry.is_symlink():
            shutil.rmtree(entry)
        else:
            entry.unlink()
    return "fresh"


def mark_copy_done(into, key):
    write_atomically(pathlib.Path(into) / COPY_DONE, key + "\n")


def copy_tree(source, into):
    """拷 COPIED_DIRECTORIES 与 COPIED_FILES；不保留修改时刻（往副本里拷源文件的纪律）。目录里除开拷标记外已有东西、缺一份、或拷完清单里还有指到副本外的 path，抛 CopyError。
    先写开拷标记；完成标记不在这里写（还要改文件），由调用方改完之后 mark_copy_done。"""
    source, into = pathlib.Path(source), pathlib.Path(into)
    check_source_is_complete(source)
    into.mkdir(parents=True, exist_ok=True)
    if any(entry.name != COPY_BEGUN for entry in into.iterdir()):
        raise CopyError(f"工作目录不空：{into}")
    write_atomically(into / COPY_BEGUN, "e203 副本拷到一半的标记：没有 .e203-copy-done 就是没拷完，可清\n")
    for name in COPIED_DIRECTORIES:
        shutil.copytree(source / name, into / name, copy_function=shutil.copyfile,
                        ignore=shutil.ignore_patterns("target"))
    for name in COPIED_FILES:
        (into / name).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source / name, into / name)
    problems = manifest_paths_not_in(into)
    if problems:
        raise CopyError("拷完的副本里清单指到副本外：" + "；".join(problems))


def check_anchors(tree):
    files = read_tree(tree, needed_paths())
    bad = 0
    for edit in edits():
        text = files[edit.file]
        try:
            position = locate(text, edit)
            print(f"name=anchor id={edit.ident} file={edit.file} line={text.count(chr(10), 0, position) + 1} hits=1")
        except AnchorError as error:
            bad += 1
            print(f"name=anchor id={edit.ident} file={edit.file} error={error}")
    return 3 if bad else 0


def build(args):
    kind, source, into = args["--build"], args["--source"], args["--into"]
    if kind not in ("counts", "timing"):
        print("--build 只认 counts 或 timing", file=sys.stderr)
        return 2
    files = read_tree(source, needed_paths())
    try:
        changed = transform(files, kind, edits())
    except AnchorError as error:
        print(f"锚点不是恰好命中一次，一个字节没写：{error}", file=sys.stderr)
        return 3
    diff_text = unified(files, changed)
    mixed = check_builds_are_not_mixed(kind, diff_text)
    if mixed:
        print(mixed, file=sys.stderr)
        return 3
    try:
        check_source_is_complete(source)
        key = copy_key(source, kind, diff_text)
        state = prepare_copy(source, into, key)
        if state == "fresh":
            copy_tree(source, into)
            for path, text in changed.items():
                if text != files[path]:
                    write_atomically(pathlib.Path(into) / path, text)
    except CopyError as error:
        print(f"副本拷不全：{error}", file=sys.stderr)
        return 2
    if "--diff-out" in args:
        write_atomically(args["--diff-out"], diff_text)
    if state == "fresh":
        mark_copy_done(into, key)
    print(f"name=counters_applied build={kind} edits={len(edits())} files={sum(1 for p in changed if changed[p] != files[p])} diff_lines={diff_text.count(chr(10))} copy={state}")
    return 0


def fake_tree():
    """一棵小的假树：每个改动的函数头、锚点、函数尾按真实锚点原样排好，函数体里的次序照真实的（判之前、固定点、准入之后）。"""
    files = {}
    for edit in edits():
        files.setdefault(edit.file, "")
    group = {}
    for edit in edits():
        anchors = group.setdefault((edit.file, edit.head, edit.end), [])
        if edit.anchor not in anchors:
            anchors.append(edit.anchor)
    for key, anchors in list(group.items()):
        group[key] = [a for a in anchors if not any(a != b and a in b for b in anchors)]
    for (path, head, end), anchors in group.items():
        body = "".join(anchors)
        files[path] += head + body + end.lstrip("\n")
    judge_key = (T, HEAD_JUDGE, FN_END)
    if judge_key in group:
        files[T] = files[T].replace(
            "    let settled = settle_the_allocation_record_tree(\n",
            "    if units_landing_is_judged {\n    }\n    let settled = settle_the_allocation_record_tree(\n")
    files["crates/singlefs-verification/Cargo.toml"] = f"[dev-dependencies]\nsinglefs-core = {{ path = \"../singlefs-core\", {HARNESS_FEATURES_FROM} }}\n"
    return files


def write_fake_source(root):
    """一棵小的假仓根：假树的文件，加上副本要拷的整套（工作区清单、锁、cargo 配置、线程登记、prune-server）。
    入口脚本的自证拿它跑真的 --build（不拷真仓、不编）。"""
    root = pathlib.Path(root)
    extra = {
        "Cargo.toml": '[workspace]\nmembers = [\n    "crates/singlefs-verification",\n]\nexclude = ["prune-server"]\n',
        "Cargo.lock": "",
        ".cargo/config.toml": "[env]\n",
        ".claude/heavy-thread-default": "31  # 理由\n",
        "prune-server/Cargo.toml": "[package]\n",
        "prune-server/src/lib.rs": "",
    }
    for relative, text in {**fake_tree(), **extra}.items():
        (root / relative).parent.mkdir(parents=True, exist_ok=True)
        (root / relative).write_text(text, encoding="utf-8")


HISTORIES = pathlib.Path(__file__).with_name("e203_admission_scan_cost_histories.rs")


def the_intent_scale_is_read_between_the_write_and_the_unlink(histories_text):
    """历史用例里 `measure_an_unlink` 的体：先建 Y、写 Y，再取意图那次的规模读数，最后 unlink；`measure` 里不再另取意图那次的读数。"""
    head = "\nfn measure_an_unlink("
    if histories_text.count(head) != 1:
        return False
    start = histories_text.index(head)
    body = histories_text[start:histories_text.index("\n}\n", start)]
    measure_head = "\nfn measure(build: &str"
    if histories_text.count(measure_head) != 1:
        return False
    measure_start = histories_text.index(measure_head)
    measure_body = histories_text[measure_start:histories_text.index("\n}\n", measure_start)]
    reading = 'state_text(build, point, state, "unlink_intent", round, pool)'
    steps = ("create_inodes(pool, 1)[0];", "overwrite_inode(pool, inode);", reading, "unlink(pool, inode);")
    if any(body.count(step) != 1 for step in steps):
        return False
    positions = [body.index(step) for step in steps]
    return positions == sorted(positions) and '"unlink_intent", round, pool)' not in measure_body


def selftest(histories_path=HISTORIES):
    import re
    failures = []

    def expect(ok, what):
        print(f"name=selftest_case case={what} result={'ok' if ok else 'FAILED'}")
        if not ok:
            failures.append(what)

    files = fake_tree()
    counts = transform(files, "counts", edits())
    timing = transform(files, "timing", edits())
    expect(all(counts[p] != files[p] for p in (T, A)), "counts_changes_the_files")
    expect("pub mod e203_scan_counts" in counts[T] and "Instant" not in counts[T], "counts_has_no_timing")
    expect("pub mod e203_scan_timing" in timing[T] and "Instant" in timing[T], "timing_has_the_timing_module")
    expect("e203_scan::steps(" not in unified(files, timing) and "e203_scan::call(" not in unified(files, timing),
           "timing_has_no_count_call_sites")
    expect("e203_scan::timer(" not in unified(files, counts), "counts_has_no_timer_call_sites")
    expect(check_builds_are_not_mixed("timing", unified(files, counts)) is not None, "mixed_diff_is_found_in_timing")
    expect(check_builds_are_not_mixed("counts", unified(files, timing)) is not None, "mixed_diff_is_found_in_counts")
    harness = "crates/singlefs-verification/Cargo.toml"
    expect("lookup-work-counters" in counts[harness], "counts_keeps_lookup_work_counters")
    expect("lookup-work-counters" not in timing[harness], "timing_drops_lookup_work_counters")
    for name, tree in (("counts", counts), ("timing", timing)):
        text = tree[T]
        between = text.index(f"{P}segment({seg('BetweenTheTwoJudgements')})")
        expect(text.index("if units_landing_is_judged {") < between < text.index("let settled = settle_the_allocation_record_tree("),
               f"{name}_segment_switch_sits_between_the_landing_judgement_and_the_fixed_point")
        caller = text.index(f"{P}segment({seg('CallerBeforeTheJudgement')})")
        expect(text.index("refuse_while_a_publish_is_frozen(allocator)?;") < caller, f"{name}_caller_segment_opens_after_the_first_refusal")
    # 计时守卫的范围（登记第十二节修订 1）：每个守卫都要归类；span 的开与关配对、先开后关；归口守卫同样
    guarded = [edit.ident for edit in edits() if edit.timing and ("timer(" in edit.timing or "timer_attributed(" in edit.timing)]
    expect(all(ident in GUARD_SCOPE for ident in guarded), "every_timing_guard_declares_its_scope")
    expect(all(ident in guarded for ident in GUARD_SCOPE), "every_declared_scope_is_a_real_timing_guard")
    expect(len({edit.ident for edit in edits()}) == len(edits()), "edit_names_are_unique")
    expect(any(scope == "span" for scope in GUARD_SCOPE.values()), "some_guards_are_statement_spans")
    timing_text, counts_text = timing[T] + timing[A], counts[T] + counts[A]
    for tree_name, tree_text, has_timer in (("timing", timing, True), ("counts", counts, False)):
        joined = "".join(tree_text[path] for path in sorted(tree_text))
        for name, (timer, attribution) in sorted(SPAN_NAMES.items()):
            if timer and has_timer:
                opened, closed = f"let _e203_timer_{name} =", f"drop(_e203_timer_{name});"
                expect(joined.count(opened) == 1 and joined.count(closed) == 1 and joined.index(opened) < joined.index(closed),
                       f"{tree_name}_timer_span_{name}_opens_then_closes")
            if attribution:
                opened, closed = f"let _e203_attribution_{name} =", f"drop(_e203_attribution_{name});"
                expect(joined.count(opened) == 1 and joined.count(closed) == 1 and joined.index(opened) < joined.index(closed),
                       f"{tree_name}_attribution_span_{name}_opens_then_closes")
    # 守卫散在计时构建改的每一份文件里（不止 transaction.rs 与 allocator.rs 两份）：逐文件数。
    # 期望数 = 这份文件里各 edit 的计时文字含的 `let _e203_timer` 个数；实际数 = 计时构建里这份文件的个数，扣掉模块与分配器助手自带的。
    timing_paths = sorted({edit.file for edit in edits() if edit.timing})
    added_by_the_helpers = {T: module_text("timing").count("let _e203_timer"), A: ALLOCATOR_HELPERS.count("let _e203_timer")}
    guards_in_the_files = {}
    for path in timing_paths:
        wanted = sum(edit.timing.count("let _e203_timer") for edit in edits() if edit.file == path and edit.timing)
        found = timing[path].count("let _e203_timer") - added_by_the_helpers.get(path, 0)
        guards_in_the_files[path] = (wanted, found)
    print("name=timing_guards_by_file " + " ".join(f"{path}={wanted}/{found}" for path, (wanted, found) in sorted(guards_in_the_files.items())))
    expect(len(guarded) > 0 and all(wanted == found for wanted, found in guards_in_the_files.values())
           and any("let _e203_timer_" in timing[path] for path in timing_paths),
           "timing_tree_has_the_statement_level_guards")
    # 清单里的每一行（清单外归口那一行除外）都有计数站点和计时站点：第一段有 12 行没落地时这两格会红
    counted_rows = [camel for camel, _, kind in ROWS if kind != "other"]
    for camel in counted_rows:
        expect(any(f"Row::{camel}" in (edit.counts or "") for edit in edits()), f"row_{camel}_has_a_counting_site")
    # 计时站点：有显式守卫的行；两处例外各有理由：s5c（照抄下段节点）的时间在 s5 的语句守卫里，s23 的两份全版索引只经归口守卫计时
    timed_through_another_row = {"FilesOtherThanTheFirstLowerNodeCopy": "FilesOtherThanTheFirstPlan",
                                 "ReleaseJudgementVersionIndexes": "attribution"}
    for camel in counted_rows:
        if camel in timed_through_another_row:
            via = timed_through_another_row[camel]
            ok = (any(f"attribute({P}Row::{camel})" in (edit.timing or "") for edit in edits()) and any("timer_attributed(" in (edit.timing or "") for edit in edits())
                  if via == "attribution" else any(f"timer({P}Row::{via})" in (edit.timing or "") for edit in edits()))
        else:
            ok = any((f"timer({P}Row::{camel})" in (edit.timing or "") or (f"Row::{camel}" in (edit.timing or "") and "timer(" in (edit.timing or "")))
                     for edit in edits())
        expect(ok, f"row_{camel}_has_a_timing_site")
    # 计数站点要记步数：每一行（清单外归口那一行除外）在计数构建里有 step / steps / contains_counted 点名它，或经归口记步（attribute 它）。
    # 只记 scale 或只记调用次数的那一行在这里红（第一段 s1、s24 只记了 scale，步数恒为 0）。
    steps_forms = ("steps(", "step(", "contains_counted(", "step_to_the_hit(", "steps_to_the_hit(")
    for camel in counted_rows:
        explicit = any(f"{P}{form}{P}Row::{camel}" in (edit.counts or "") for edit in edits() for form in steps_forms)
        attributed = any(f"attribute({P}Row::{camel})" in (edit.counts or "") for edit in edits())
        expect(explicit or attributed, f"row_{camel}_has_a_steps_site")
    # 整份拷那两行（s1、s24）：步数就是这一份要拷的元素数（登记读法第 3 条），插计数后的计数构建里各恰好一处（变异 c13、c14）
    counts_joined = "".join(counts[path] for path in sorted(counts))
    for camel in ("WholeAllocatorCopyBeforeTheJudgement", "WholeAllocatorCopyInASettlingRound"):
        steps_of_the_copy = f"{P}steps({P}Row::{camel}, e203_copied_elements);"
        expect(counts_joined.count(steps_of_the_copy) == 1
               and "let e203_copied_elements = allocator.e203_whole_copy_elements();" in counts_joined,
               f"row_{camel}_records_the_copied_elements_as_steps")
    # 登记第七节 ③2 的规模读数（修订 3 ②）：每一条在计数构建里恰好一处、读的是这一版的那张表，入口那几条在落点判那一段换进之后、
    # 出口那几条在换回 Outside 之前；计时构建一条都不插（变异 c17–c45）。期望的文字逐字写在这里，不从读数表里生成。
    readings_expected = {
        "entry": (HEAD_JUDGE, f"{P}segment({seg('BeforeTheLandingJudgement')});",
                  "let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator", (
            'delta("entry_accounting_rows", e203_version.accounting_entries.len());',
            'delta("entry_watermark_rows_before_its_txg", e203_version.accounting_entries.iter().filter(|entry| entry.statistic == '
            'STATISTIC_INODE_WATERMARK && entry.generation < e203_version.root.checkpoint_txg).count());',
            'delta("entry_accounting_tree_keys", e203_version.accounting_tree.shape.keys_in_order().len());',
            'delta("entry_accounting_tree_nodes", e203_version.accounting_tree.shape.nodes().len());',
            'delta("entry_deadlist_tree_keys", e203_version.deadlist_tree.shape.keys_in_order().len());',
            'delta("entry_deadlist_tree_nodes", e203_version.deadlist_tree.shape.nodes().len());',
            'delta("entry_central_mapping_tree_keys", e203_version.central_mapping_tree.shape.keys_in_order().len());',
            'delta("entry_central_mapping_tree_nodes", e203_version.central_mapping_tree.shape.nodes().len());',
            'delta("entry_allocation_record_tree_nodes", e203_version.allocation_record_tree.node_set().len());',
            'delta("entry_units", e203_version.units.len());',
            'delta("entry_mapped_units", e203_version.mapped_units.len());',
            'delta("entry_files_other_than_the_first", e203_version.files_other_than_the_first.len());',
            'delta("entry_deadlist_entries", e203_version.deadlist_entries.len());',
            'delta("entry_kept_mapping_entries", e203_version.mapping_entries_kept_for_the_snapshots.len());',
            'delta("entry_position_of_the_inode_whose_size_changes", deletion_and_lineage.step.inode_whose_size_changes().map_or(0, |e203_inode| {'
            ' let e203_records = || e203_version.inode_leaf_containers.iter().flat_map(|container| container.contents.records.iter());'
            ' if e203_records().any(|record| record.inode == e203_inode.0) {'
            ' e203_records().filter(|record| record.inode < e203_inode.0).count() + 1 } else { e203_records().count() } }));',
            'delta("entry_instance_table_chain_replaced", match &plan.instance_table { InstanceTablePlan::Rewrite(e203_rewrite) => '
            'e203_rewrite.replaced_chain.len(), InstanceTablePlan::Carry(_) => 0 });',
            'delta("entry_cluster_segments", allocator.cluster_segments().len());',
        )),
        "exit": (HEAD_AFTER_A_CHECK, f"{P}segment({seg('CallerBeforeTheJudgement')});", f"{P}segment({seg('Outside')});", (
            'delta("exit_accounting_tree_keys", e203_output.accounting_tree.shape.keys_in_order().len());',
            'delta("exit_accounting_tree_nodes", e203_output.accounting_tree.shape.nodes().len());',
            'delta("exit_deadlist_tree_keys", e203_output.deadlist_tree.shape.keys_in_order().len());',
            'delta("exit_deadlist_tree_nodes", e203_output.deadlist_tree.shape.nodes().len());',
            'delta("exit_central_mapping_tree_keys", e203_output.central_mapping_tree.shape.keys_in_order().len());',
            'delta("exit_central_mapping_tree_nodes", e203_output.central_mapping_tree.shape.nodes().len());',
            'delta("exit_mapped_units", e203_output.mapped_units.len());',
            'delta("exit_allocation_record_tree_nodes", e203_output.allocation_record_tree.node_set().len());',
            'delta("exit_deadlist_entries", e203_output.deadlist_entries.len());',
            'delta("exit_container_tree_units", e203_output.tombstone_containers.len() + e203_output.tombstone_tree_index_nodes.shape.'
            'positions_in_bump_order().len() + e203_output.lineage_containers.len() + e203_output.lineage_tree_index_nodes.shape.'
            'positions_in_bump_order().len() + e203_output.retirement_containers.len() + e203_output.retirement_tree_index_nodes.shape.'
            'positions_in_bump_order().len());',
            'delta("exit_extent_tree_nodes", e203_output.extent_tree.lower_nodes.len() + e203_output.extent_tree.upper_nodes.len() + '
            'e203_output.files_other_than_the_first.iter().map(|file| file.extent_lower_nodes.len()).sum::<usize>());',
        )),
    }
    for where, (head, after_this, before_this, lines) in readings_expected.items():
        start = counts[T].index(head)
        body = counts[T][start:counts[T].index(FN_END, start + len(head))]
        for line in lines:
            key = line[len('delta("'):line.index('",')]
            written = f"{P}{line}"
            placed = (body.count(written) == 1 and body.count(after_this) == 1 and body.count(before_this) == 1
                      and body.index(after_this) < body.index(written) < body.index(before_this))
            expect(placed and f'"{key}"' not in timing[T], f"scale_reading_{key}_is_read_at_the_{where}_in_the_counts_build_only")
        # 不量的那一段停掉读数：这一组规模读数整段包在「读数没停」里，包的那一句在第一条读数之前、只有一处，计时构建里没有（变异 c68）
        wrap = f"    if !{P}readings_paused() {{\n"
        first_reading = f"{P}{lines[0]}"
        expect(body.count(wrap) == 1 and body.index(wrap) < body.index(first_reading) and wrap not in timing[T],
               f"scale_readings_at_the_{where}_are_skipped_while_the_readings_are_paused")
    # 停读数的开关（不量的那一段提速）：两种构建都有这一对函数、问的是同一个线程局部的布尔（变异 c67）；计数构建每个记读数的入口先问停没停，
    # 计时构建的守卫停了不计时（变异 c69、c70）；换段那一个函数不问（记录照旧开与收，判了几次照数）。期望的文字逐字写在这里，不从模板里生成。
    paused_return = "if readings_paused() {\n            return;\n        }\n"
    for build_name, module in (("counts", module_text("counts")), ("timing", module_text("timing"))):
        # 用正则写，免得本脚本源码里同一句出现两次（变异表按原文在本脚本里恰好命中一次找锚点）
        expect(re.search(r"pub fn pause_readings\(paused: bool\) \{\s+READINGS_PAUSED\.with\(\|cell\| cell\.set\(paused\)\);\s+\}", module)
               is not None
               and re.search(r"pub fn readings_paused\(\) -> bool \{\s+READINGS_PAUSED\.with\(std::cell::Cell::get\)\s+\}", module) is not None,
               f"{build_name}_module_has_the_switch_that_pauses_the_readings")
        segment_body = module[module.index("    pub fn segment(next: Segment) {"):module.index("    pub fn round_begin() {")]
        expect("readings_paused" not in segment_body, f"{build_name}_segment_switch_opens_and_closes_records_while_paused")
    counts_module, timing_module = module_text("counts"), module_text("timing")
    for function in ("fn with_open(change: impl FnOnce(&mut Open, usize, u32)) {\n        ", "pub fn call_attributed() {\n        ",
                     "pub fn steps_attributed(count: usize) {\n        ", "pub fn query_attributed(index_entries: u64) {\n        ",
                     "pub fn delta(key: &'static str, value: usize) {\n        "):
        expect(counts_module.count(function + paused_return) == 1,
               f"counts_{function.split('(')[0].split()[-1]}_records_nothing_while_the_readings_are_paused")
    expect("    pub fn delta_if_attributed(row: Row, key: &'static str, value: usize) {\n        if !readings_paused() && attributed_row() == row {\n"
           in counts_module, "counts_delta_if_attributed_records_nothing_while_the_readings_are_paused")
    expect("    pub fn timer(row: Row) -> TimerGuard {\n        if readings_paused() {\n            return TimerGuard { active: false };\n        }\n"
           in timing_module, "timing_guard_is_inactive_while_the_readings_are_paused")
    # 修订 4：十行在每次调用那一刻读它要扫的那张表的长度。每条读数在计数构建里它那个函数体里恰好一处、在扫那一句之前（与它之后的那一句之前），
    # 计时构建里没有；读的只是字段长度与本地的循环边界，不调插了计数的函数（变异 c46 起）。期望的文字逐字写在这里，不从改动表里生成。
    ART, EXT, CHANGED = CORE + "allocation_record_tree.rs", CORE + "extent_tree.rs", "\npub fn nodes_whose_contents_changed(\n"
    make_room, settle_round = "\nfn make_room_for_record_on_device(\n", f"{P}round_begin();"
    call_site_readings = (
        ("s6_find", T, "    pub fn file_other_than_the_first(\n", METHOD_END, None, ".find(|file| {",
         f"{P}scale_to_the_hit({P}Row::FileOtherThanTheFirstLinearFind, self.files_other_than_the_first.len());"),
        ("s8_path", EXT, "\npub fn plan_the_extent_upper_segment(\n", FN_END, "canonical_upper_segment_of_the_inodes_with_an_entry(",
         "let covers_a_changed_inode",
         f"{P}scale({P}Row::ExtentUpperSegmentPlan, inodes_with_an_entry_after_this_publish.len() * (1 + positions_after_this_publish.iter()"
         ".map(|position| usize::from(position.level)).max().unwrap_or(0)));"),
        ("s8_contains", EXT, "\npub fn plan_the_extent_upper_segment(\n", FN_END, "if covers_a_changed_inode(position) || !{",
         f"{P}contains_counted({P}Row::ExtentUpperSegmentPlan, previous_upper_nodes, position) }}",
         f"{P}scale_to_the_hit({P}Row::ExtentUpperSegmentPlan, previous_upper_nodes.len());"),
        ("s8_replaced_filter", EXT, "\npub fn plan_the_extent_upper_segment(\n", FN_END, None, "let replaced_previous_positions = previous_upper_nodes",
         f"{P}scale({P}Row::ExtentUpperSegmentPlan, previous_upper_nodes.len());"),
        ("s12_find", T, "    pub fn unit(&self, identity: TransactionUnit) -> &PublishedUnit {\n", METHOD_END, None, ".find(|unit| {",
         f"{P}scale_to_the_hit({P}Row::RootHeaderLinearFind, self.units.len());"),
        ("s16_plan", ART, "\npub fn plan_the_tree_after_this_publish(\n", FN_END, None, "let nodes = nodes_after",
         f"{P}scale({P}Row::AllocationRecordTreePlan, nodes_after.len() + previous_nodes.len());"),
        ("s16_intersection", T, HEAD_SETTLE, FN_END, settle_round, "let rewritten_among_the_nodes_after",
         f"{P}scale({P}Row::AllocationRecordTreePlan, rewritten.len() + nodes_after.len());"),
        ("s25_record_for", A, "    pub fn record_for(\n", METHOD_END, None, ".find(|record| {",
         f"{P}scale_to_the_hit({P}Row::AllocatorReleaseLinearFind, self.records.len());"),
        ("s25_release", A, "    pub fn release_leaving_the_record_allocated_on(\n", METHOD_END, None, "let record = self",
         f"{P}scale_to_the_hit({P}Row::AllocatorReleaseLinearFind, self.records.len());"),
        ("s26_candidates", A, "    pub fn lowest_user_data_slot_outside_the_region(\n", METHOD_END, f"{P}call({P}Row::UserDataSlotLinearScan);",
         "while candidate + 1 < end {",
         f"{P}scale_to_the_hit({P}Row::UserDataSlotLinearScan, usize::try_from(end.saturating_sub(candidate) / 2)"
         '.expect("候选数装得进 usize") * (1 + cluster_segments.len()));'),
        ("s27_filter", A, make_room, FN_END, None, "let intersecting_record_slots",
         f"{P}scale({P}Row::RecordIntersectionFilter, records.len());\n    let intersecting_record_slots"),
        ("s27_find", A, make_room, FN_END, None, "let existing = records",
         f"{P}scale_to_the_hit({P}Row::RecordIntersectionFilter, records.len());"),
        ("s27_retain", A, make_room, FN_END, "let existing = records", "records.retain(|record| {",
         f"{P}scale({P}Row::RecordIntersectionFilter, records.len());\n            records.retain"),
        ("s28_segments", A, "    pub fn lowest_empty_segment_outside_the_region(\n", METHOD_END, "let per_segment =", "(0..full_segments)",
         f"{P}scale_to_the_hit({P}Row::EmptySegmentLinearScan, self.allocated.len() / per_segment);"),
        ("s29_candidates", A, "    pub fn lowest_commit_generated_fallback_slot_outside_the_region(\n", METHOD_END,
         f"{P}call({P}Row::FallbackSlotLinearScan);", "while candidate + footprint.slots() <= unit_area_end {",
         f"{P}scale_to_the_hit({P}Row::FallbackSlotLinearScan, usize::try_from(unit_area_end.checked_sub(candidate + footprint.slots())"
         '.map_or(0, |e203_room| e203_room / footprint.alignment_in_slots() + 1) * (1 + footprint.slots())).expect("上界装得进 usize"));'),
        ("s30_records", ART, "\npub fn records_of_each_leaf(\n", FN_END, None, "let mut by_leaf",
         f"{P}scale({P}Row::RecordsOfEachLeaf, records.len());"),
        ("s30_leaves", ART, CHANGED, FN_END, "before.keys().chain(after.keys()).copied().collect();", f"{P}steps({P}Row::RecordsOfEachLeaf, leaves.len());",
         f"{P}scale({P}Row::RecordsOfEachLeaf, leaves.len());"),
    )
    for name, path, head, end, after_this, before_this, written in call_site_readings:
        start = counts[path].index(head)
        body = counts[path][start:counts[path].index(end, start + len(head)) + len(end)]
        low = body.index(after_this) + len(after_this) if after_this and body.count(after_this) == 1 else (0 if not after_this else -1)
        high = body.index(before_this) if body.count(before_this) == 1 else -1
        at = body.find(written)
        placed = body.count(written) == 1 and 0 <= low <= at and at < high
        expect(placed and written not in timing[path], f"call_site_length_{name}_is_read_before_the_scan_in_the_counts_build_only")
    # 走到命中为止的那一部分的步记进 steps_to_the_hit（全表扫的照旧记 step / steps）：每行两种记法各几处，逐行写死（变异 c46 起）
    walks_to_the_hit = {"FileOtherThanTheFirstLinearFind": (1, 0), "RootHeaderLinearFind": (1, 0), "AllocatorReleaseLinearFind": (2, 0),
                        "UserDataSlotLinearScan": (2, 0), "EmptySegmentLinearScan": (1, 0), "FallbackSlotLinearScan": (2, 0),
                        "RecordIntersectionFilter": (1, 2), "ExtentUpperSegmentPlan": (0, 1), "AllocationRecordTreePlan": (0, 0),
                        "RecordsOfEachLeaf": (0, 0)}
    every_counts_file = "".join(counts[path] for path in sorted(counts))
    for camel, (to_the_hit, plain) in walks_to_the_hit.items():
        found = (every_counts_file.count(f"{P}step_to_the_hit({P}Row::{camel});"), every_counts_file.count(f"{P}step({P}Row::{camel});"))
        expect(found == (to_the_hit, plain), f"row_{camel}_counts_the_walk_to_the_hit_apart")
    # s25 两处查找（`record_for` 与 `release_leaving_the_record_allocated_on` 每块盘一次）各记一次调用、各有一个计时守卫：
    # 调用数是 B8 那一行 c25 × lg(R_a) 的 c25，只记一处时少一半（变异 c71、c72）
    counted_call_of_the_release_find = f"{P}call({P}Row::AllocatorReleaseLinearFind);"
    timer_of_the_release_find = f"{P}timer({P}Row::AllocatorReleaseLinearFind);"
    release_find_sites = (
        ("    pub fn record_for(\n", ".find(", counts, counted_call_of_the_release_find,
         "counts_release_find_in_record_for_is_marked_before_the_find"),
        ("    pub fn record_for(\n", ".find(", timing, timer_of_the_release_find,
         "timing_release_find_in_record_for_is_marked_before_the_find"),
        ("    pub fn release_leaving_the_record_allocated_on(\n", "let record = self", counts, counted_call_of_the_release_find,
         "counts_release_find_in_release_is_marked_before_the_find"),
        ("    pub fn release_leaving_the_record_allocated_on(\n", "let record = self", timing, timer_of_the_release_find,
         "timing_release_find_in_release_is_marked_before_the_find"),
    )
    for site_head, site_find, tree, needle, case_name in release_find_sites:
        site_start = tree[A].index(site_head)
        site_body = tree[A][site_start:tree[A].index(METHOD_END, site_start + len(site_head)) + len(METHOD_END)]
        placed = (site_body.count(needle) == 1 and site_body.count(site_find) == 1
                  and site_body.index(needle) < site_body.index(site_find))
        expect(placed, case_name)
    expect(every_counts_file.count(f"{P}call({P}Row::AllocatorReleaseLinearFind);") == 2,
           "counts_build_records_a_call_at_both_release_finds")
    expect("steps_to_the_hit(row, walked.map_or(haystack.len(), |index| index + 1));" in counts[T]
           and "open.steps_to_the_hit[cell] += count;" in counts[T] and "open.steps[cell] += count;" in counts[T],
           "a_step_to_the_hit_is_counted_in_both_the_steps_and_the_part_to_the_hit")
    # deadlist 树走零根那一支时也记一条进出的 key 数（每轮三棵码 2 树各一条，模型按三条读；变异 c15）
    zero_root_text = counts[CORE + "code_two_tree.rs"]
    zero_root_start = zero_root_text.index(HEAD_ZERO_ROOT)
    zero_root_body = zero_root_text[zero_root_start:zero_root_text.index(FN_END, zero_root_start + len(HEAD_ZERO_ROOT))]
    expect('"code_two_keys_changed", previous.keys_in_order().len())' in zero_root_body
           and "steps_attributed(previous.nodes().len())" in zero_root_body
           and zero_root_body.index('"code_two_keys_changed"') > zero_root_body.index("return plan_the_tree_after_this_publish("),
           "the_deadlist_zero_root_branch_records_the_keys_changed")
    # 历史用例：意图那次的规模读数在 Y 建好、写好之后、unlink 之前取（第一段在建 Y 之前取，别的文件数与聚簇段数各少一，停机条款第 2 条打中 98 格；变异 c16）
    histories_text = pathlib.Path(histories_path).read_text(encoding="utf-8")
    expect(the_intent_scale_is_read_between_the_write_and_the_unlink(histories_text),
           "the_unlink_intent_scale_is_read_after_the_file_to_unlink_is_written")
    # 副本要编得起来：缺哪一份就拒，拷出来的清单不许指到副本外（变异 c4）
    import tempfile
    with tempfile.TemporaryDirectory() as directory:
        def make_source(name):
            root = pathlib.Path(directory) / name
            for relative, text in (
                ("Cargo.toml", '[workspace]\nmembers = [\n    "crates/a",\n]\nexclude = ["prune-server"]\n'),
                ("Cargo.lock", ""),
                (".cargo/config.toml", "[env]\n"),
                (".claude/heavy-thread-default", "31  # 理由\n"),
                ("crates/a/Cargo.toml", '[dependencies]\nprune-server = { path = "../../prune-server" }\n'),
                ("crates/a/src/lib.rs", ""),
                ("prune-server/Cargo.toml", "[package]\n"),
                ("prune-server/src/lib.rs", ""),
                ("prune-server/target/skip-me", ""),
            ):
                (root / relative).parent.mkdir(parents=True, exist_ok=True)
                (root / relative).write_text(text, encoding="utf-8")
            return root
        whole = make_source("whole")
        try:
            copy_tree(whole, pathlib.Path(directory) / "copy-whole")
            copied = pathlib.Path(directory) / "copy-whole"
            expect(all((copied / name).exists() for name in COPIED_DIRECTORIES + COPIED_FILES), "copy_has_everything_a_build_needs")
            expect(not (copied / "prune-server" / "target").exists(), "copy_leaves_the_build_directory_out")
        except CopyError as error:
            print(f"copy_error={error}")
            expect(False, "copy_has_everything_a_build_needs")
        for index, name in enumerate(COPIED_DIRECTORIES + COPIED_FILES):
            source = make_source(f"missing-{index}")
            target = source / name
            if target.is_dir():
                shutil.rmtree(target)
            else:
                target.unlink()
            try:
                copy_tree(source, pathlib.Path(directory) / f"copy-missing-{index}")
                expect(False, f"copy_is_refused_when_{name.strip('.').replace('/', '_')}_is_missing")
            except CopyError:
                expect(True, f"copy_is_refused_when_{name.strip('.').replace('/', '_')}_is_missing")
        left_out = pathlib.Path(directory) / "copy-whole"
        shutil.rmtree(left_out / "prune-server")
        expect(any("prune-server" in problem for problem in manifest_paths_not_in(left_out)), "a_path_dependency_outside_the_copy_is_found")

        # 续跑：同一条命令被打断之后重起，认得出上一次的半拷、拷完、键变了与不是自己拷的
        def trying(action):
            try:
                return action()
            except CopyError:
                return "refused"

        def finish(source, into, key):
            try:
                copy_tree(source, into)
                mark_copy_done(into, key)
            except CopyError:
                pass  # 前面的 prepare_copy 拒了（弄坏开关下）：后面读结果的那一格会红，这里不让整份自证崩掉

        resume = make_source("resume")
        resume_into = pathlib.Path(directory) / "copy-resume"
        key = copy_key(resume, "counts", "diff")
        expect(trying(lambda: prepare_copy(resume, resume_into, key)) == "fresh", "an_empty_directory_is_copied_fresh")
        finish(resume, resume_into, key)
        expect(trying(lambda: prepare_copy(resume, resume_into, key)) == "reused", "a_finished_copy_with_the_same_key_is_reused")
        (resume / "crates/a/src/lib.rs").write_text("// changed\n", encoding="utf-8")
        changed_key = copy_key(resume, "counts", "diff")
        expect(changed_key != key, "the_key_follows_the_content_of_a_copied_file")
        expect(copy_key(resume, "timing", "diff") != changed_key and copy_key(resume, "counts", "other diff") != changed_key,
               "the_key_follows_the_build_kind_and_the_diff")
        (resume_into / "stale-file").write_text("", encoding="utf-8")
        expect(trying(lambda: prepare_copy(resume, resume_into, changed_key)) == "fresh" and not (resume_into / "stale-file").exists()
               and not (resume_into / COPY_DONE).exists(), "a_finished_copy_whose_inputs_changed_is_cleared")
        finish(resume, resume_into, changed_key)
        expect((resume_into / "crates/a/src/lib.rs").read_text(encoding="utf-8") == "// changed\n", "a_copy_after_the_clearing_has_the_new_content")
        half = pathlib.Path(directory) / "copy-half"
        trying(lambda: prepare_copy(resume, half, changed_key))
        real_copyfile, copied = shutil.copyfile, []

        def cut_off_after_two_files(source_file, target_file, *rest, **named):
            copied.append(source_file)
            if len(copied) > 2:
                raise KeyboardInterrupt  # 被杀：不是 OSError，copytree 不会吞
            return real_copyfile(source_file, target_file, *rest, **named)

        shutil.copyfile = cut_off_after_two_files
        try:
            copy_tree(resume, half)
        except KeyboardInterrupt:
            pass
        finally:
            shutil.copyfile = real_copyfile
        expect((half / COPY_BEGUN).exists() and not (half / COPY_DONE).exists() and len(list(half.iterdir())) > 1,
               "a_cut_off_copy_leaves_the_begun_marker_and_no_done_marker")
        expect(trying(lambda: prepare_copy(resume, half, changed_key)) == "fresh", "a_half_copy_is_cleared_and_copied_again")
        finish(resume, half, changed_key)
        expect(all((half / name).exists() for name in COPIED_DIRECTORIES + COPIED_FILES) and (half / COPY_DONE).exists(),
               "a_half_copy_ends_up_complete_after_the_second_try")
        unmarked = pathlib.Path(directory) / "copy-unmarked"
        trying(lambda: prepare_copy(resume, unmarked, changed_key))
        copy_tree(resume, unmarked)
        expect(trying(lambda: prepare_copy(resume, unmarked, changed_key)) == "fresh", "a_copy_without_the_done_marker_is_not_trusted")
        cut_marker = pathlib.Path(directory) / "copy-cut-marker"
        cut_marker.mkdir()
        (cut_marker / (COPY_BEGUN + ".partial")).write_text("", encoding="utf-8")
        expect(trying(lambda: prepare_copy(resume, cut_marker, changed_key)) == "fresh" and not any(cut_marker.iterdir()),
               "a_begun_marker_cut_off_while_it_was_written_is_still_ours")
        foreign = pathlib.Path(directory) / "foreign"
        foreign.mkdir()
        (foreign / "mine.txt").write_text("not ours", encoding="utf-8")
        expect(trying(lambda: prepare_copy(resume, foreign, changed_key)) == "refused" and (foreign / "mine.txt").exists(),
               "a_directory_that_is_not_our_copy_is_refused_and_left_alone")
    # 同一个锚点在函数体里出现两次：退 3 那一格（变异 c1）
    doubled = dict(files)
    first = edits()[0]
    doubled[first.file] = doubled[first.file].replace(first.anchor, first.anchor + first.anchor, 1)
    try:
        transform(doubled, "counts", edits())
        expect(False, "an_anchor_that_hits_twice_stops_the_run")
    except AnchorError:
        expect(True, "an_anchor_that_hits_twice_stops_the_run")
    # 锚点不在：同样停
    missing = dict(files)
    missing[first.file] = missing[first.file].replace(first.anchor, "", 1)
    try:
        transform(missing, "timing", edits())
        expect(False, "a_missing_anchor_stops_the_run")
    except AnchorError:
        expect(True, "a_missing_anchor_stops_the_run")
    print(f"name=selftest_done cases_failed={len(failures)}")
    return 4 if failures else 0


def mutation_check(table):
    """对变异表逐条跑：把原文换成替换文写到临时文件，跑 --selftest，必须退非 0 才算抓到。

    每行三段（名、原文、替换文）改的是本脚本；第四段写 `histories` 的改的是历史用例 `e203_admission_scan_cost_histories.rs`，
    本脚本原样、自证经 --histories 读改过的那一份。自证一律经 --histories 指到历史用例（改过的或仓里那一份），
    临时目录里的本脚本旁边没有历史用例。"""
    import subprocess
    import tempfile
    sources = {"script": pathlib.Path(__file__).read_text(encoding="utf-8"),
               "histories": HISTORIES.read_text(encoding="utf-8")}
    missed = 0
    for line in pathlib.Path(table).read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        name, original, replacement = fields[:3]
        target = fields[3] if len(fields) == 4 else "script"
        if target == "histories":
            # 历史用例是 Rust：表里的 \n（反斜杠加 n）还原成换行，好写跨行的原文
            original, replacement = original.replace("\\n", "\n"), replacement.replace("\\n", "\n")
        # 改本脚本的那几行：原文与替换文逐字取自本脚本的源码（源码里的 \n 是反斜杠加 n 两个字符），不做转义还原
        source = sources[target]
        if source.count(original) != 1:
            print(f"name=counters_mutation id={name} result=anchor_lost hits={source.count(original)}")
            missed += 1
            continue
        with tempfile.TemporaryDirectory() as directory:
            mutated = pathlib.Path(directory) / "mutated.py"
            histories = pathlib.Path(directory) / "histories.rs"
            mutated.write_text(source.replace(original, replacement) if target == "script" else sources["script"], encoding="utf-8")
            histories.write_text(source.replace(original, replacement) if target == "histories" else sources["histories"], encoding="utf-8")
            outcome = subprocess.run([sys.executable, "-I", str(mutated), "--selftest", "--histories", str(histories)],
                                     capture_output=True, text=True, check=False)
        caught = outcome.returncode != 0
        missed += 0 if caught else 1
        print(f"name=counters_mutation id={name} result={'caught' if caught else 'missed'} exit={outcome.returncode}")
    print(f"name=counters_mutation_done missed={missed}")
    return 0 if missed == 0 else 4


def break_switch_is_allowed(argv):
    """弄坏开关只在自证里认：命令行就是 --selftest，或入口脚本的自证设了 E203_SELFTEST_MARK 指到它建的标记文件（内容对得上）。别处设了拒。"""
    if not os.environ.get("E203_RUN_BREAK") or "--selftest" in argv:
        return True
    mark = os.environ.get("E203_SELFTEST_MARK")
    try:
        return bool(mark) and pathlib.Path(mark).read_text(encoding="utf-8").strip() == "e203-selftest"
    except OSError:
        return False


def main(argv):
    if not break_switch_is_allowed(argv):
        print("  ✗ E203_RUN_BREAK 只给自证证红用，这一趟不是自证起的\n"
              "     → 怎么办：去掉 E203_RUN_BREAK 再跑；要证红就跑 E203_RUN_BREAK=<项> python3 -I e203_admission_scan_cost_counters.py --selftest", file=sys.stderr)
        return 2
    args, index = {}, 0
    while index < len(argv):
        if argv[index] in ("--selftest",):
            args[argv[index]] = True
            index += 1
        elif argv[index].startswith("--") and index + 1 < len(argv):
            args[argv[index]] = argv[index + 1]
            index += 2
        else:
            print(__doc__, file=sys.stderr)
            return 2
    if "--selftest" in args:
        return selftest(args.get("--histories", HISTORIES))
    if "--check-anchors" in args:
        return check_anchors(args["--check-anchors"])
    if "--write-fake-source" in args:
        write_fake_source(args["--write-fake-source"])
        return 0
    if "--mutation-check" in args:
        return mutation_check(args["--mutation-check"])
    if all(key in args for key in ("--build", "--source", "--into")):
        return build(args)
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
