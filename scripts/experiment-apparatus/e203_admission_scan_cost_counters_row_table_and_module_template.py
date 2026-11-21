# 这个脚本的全部文件：入口与拆出的各部分（都在同一个目录）。副本的输入键与变异检查按这张表读本脚本全文。
SCRIPT_FILE_NAMES = (
    "e203_admission_scan_cost_counters.py",
    "e203_admission_scan_cost_counters_row_table_and_module_template.py",
    "e203_admission_scan_cost_counters_insertion_edits.py",
    "e203_admission_scan_cost_counters_applying_edits_and_copying_the_tree.py",
    "e203_admission_scan_cost_counters_selftest.py",
)

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
    ("PooledFallbackByTheLeaf", "pooled_fallback_by_the_leaf", "scan"),
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
