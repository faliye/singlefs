#!/usr/bin/env python3
"""E206（发布固定点轮数的代价）量的那一路：在拷来的实现上插固定点的逐轮读数与臂的补丁，不改 E203 的编辑表。

跑前登记 research/prompts/e206-preregistration.md 第三节「每个读数的取样位置」、第五节「装置写在哪」第 1 条。

用法：
  python3 e206_fixed_point_cost_counters.py --build counts|plain --source <仓根> --into <工作目录> [--diff-out <文件>]
  python3 e206_fixed_point_cost_counters.py --check-anchors <树>      只核锚点、打每个锚点的行号，不写；锚点全对之后再不起编译地判一遍：插完的每行读数语句要落在语句层（前一个字符是 ; { }、最内层括号是 {，不在表达式、实参表、链式调用中间），改过的 .rs 括号要配平，不过退 3、打 name=insertion_problem / name=balance_problem 行（判法在 E203 脚本里）
  python3 e206_fixed_point_cost_counters.py --selftest                用一棵小的假树证每个分支（不编实现）
  python3 e206_fixed_point_cost_counters.py --mutation-check <变异表>  逐条改坏、跑自证，没红的报 missed
  python3 e206_fixed_point_cost_counters.py --write-fake-source <新目录>  写一棵小的假仓根（入口脚本的自证用）
退出码：0 成；2 用法错或文件不在；3 有锚点命中不是恰好一次（一个字节不写，登记停机条款第 1 条）；4 自证有一格不对。

--build counts：先照 e203_admission_scan_cost_counters.py 的编辑表插 E203（准入之前扫类的步代价）的计数（import 它，不改它），再插第三节的 E206 读数与两条臂的补丁，
  测试目录里写 e206_readings/mod.rs（读数行怎么打）。臂由环境变量 E206_ARM 在进程开头读一次（p1、p2a、p2b16、p2b256；对照 whole、single_step、closure_off、p2b_never），
  一份构建带全部臂。
--build plain：只拷不插，测试目录里写另一份 e206_readings/mod.rs（读数全空），给作废条款第 5 条比盘。
每个锚点在它那个函数体里必须恰好命中一次，否则退 3。副本的开拷、完成标记与续拷照 E203 脚本的 prepare_copy、mark_copy_done（同一条命令重起就是续跑）。
弄坏开关（只给自证用，平常跑时不认）：E206_RUN_BREAK=<项> 由入口自证带过来，这份脚本只认 copy-resume-off（照 E203 脚本的 E203_RUN_BREAK=copy-resume-off 转过去）：
  E206_RUN_BREAK=copy-resume-off  不认完成标记与开拷标记，非空目录一律拒
      入口自证的 killed_after_the_copy_and_run_again_completes（失败行句首 name=selftest_case case=<这个名字> result=FAILED）
"""
import importlib.util
import os
import pathlib
import re
import sys
import tempfile

E203_SCRIPT = pathlib.Path(os.path.join(os.path.dirname(os.path.realpath(__file__)), "e203_admission_scan_cost_counters.py"))
READINGS_IN_THE_COPY = "crates/singlefs-verification/tests/e206_readings/mod.rs"
BUILDS = ("counts", "plain")
E = "crate::transaction::e206_fixed_point::"


def load_e203():
    specification = importlib.util.spec_from_file_location("e203_counters_for_e206", os.path.join(os.path.dirname(os.path.realpath(__file__)), "e203_admission_scan_cost_counters.py"))
    if specification is None or not E203_SCRIPT.is_file():
        print(f"E203 的插计数脚本不在：{E203_SCRIPT}", file=sys.stderr)
        sys.exit(2)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


e203 = load_e203()
T = e203.T
FN_END = e203.FN_END
HEAD_SETTLE = e203.HEAD_SETTLE
HEAD_JUDGE = e203.HEAD_JUDGE
HEAD_ROW_SETTLE = "\nfn settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file(\n"
HEAD_ROW_CALLER = "\npub fn prepare_the_row_publish_on_a_version_without_file<Device: BlockDevice>(\n"
CODE2 = e203.CORE + "code_two_tree.rs"
HEAD_CODE_TWO_PLAN = "\npub fn plan_the_tree_after_this_publish(\n"
HEAD_ZERO_ROOT = e203.HEAD_ZERO_ROOT

# 补进 E203 模块里的只读取数：模块的字段是私有的，读数第 5 组与「这一条固定点属于第几次判」只能在模块里面读。
E203_MODULE_ANCHOR = "    /// 固定点每轮开头记一次。\n    pub fn round_begin() {\n"
E203_MODULE_ACCESSORS = r'''    /// 这一刻两道之间那一段每行累计的调用数与步数（E206（发布固定点轮数的代价）读数第 5 组在每轮开头切）。
    #[must_use]
    pub fn between_row_totals() -> Vec<(&'static str, u64, u64, u64)> {
        THREAD.with(|thread_cell| {
            let thread = thread_cell.borrow();
            let Some(open) = thread.open.as_ref() else {
                return Vec::new();
            };
            let segment = Segment::BetweenTheTwoJudgements as usize;
            ALL_ROWS
                .iter()
                .enumerate()
                .filter_map(|(row_index, row)| {
                    let position = segment * ROW_COUNT + row_index;
                    let (calls, steps, query_levels) = (open.calls[position], open.steps[position], open.query_levels[position]);
                    (calls + steps + query_levels > 0).then(|| (row.name(), calls, steps, query_levels))
                })
                .collect()
        })
    }
    /// 已收好、还没取走的记录条数：正在开着的那一条判在记录表里的下标。
    #[must_use]
    pub fn finished_records() -> usize {
        THREAD.with(|thread_cell| thread_cell.borrow().finished.len())
    }
    /// 此刻的步归口到三棵多层码 2 树的规划（同 `delta_if_attributed` 那一判）。
    #[must_use]
    pub fn attributed_to_multi_level_planning() -> bool {
        attributed_row() == Row::MultiLevelTreePlanning
    }
'''

MODULE_PART_ARMS = r'''
/// E206（发布固定点轮数的代价）只在副本里套上的读数与臂（跑前登记 research/prompts/e206-preregistration.md 第三、五节）。
pub mod e206_fixed_point {
    use super::{PublishError, TransactionUnit, ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT};
    use crate::allocation_record_tree::{
        nodes_holding_records, AllocationRecordTreeGeometry, AllocationRecordTreeNode,
        AllocationRecordTreeVersion,
    };
    use crate::allocator::{AllocationRecord, Placement};
    use std::cell::RefCell;
    use std::collections::{BTreeMap, BTreeSet, HashMap};
    use std::sync::OnceLock;

    /// 一条臂或一条对照（登记第五节「三条岔路的臂」「阳性对照」）。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Arm {
        Today,
        ReleaseChainClosure,
        WholeTreeEvery16,
        WholeTreeEvery256,
        WholeTreeEveryPublish,
        OneNodeAtATime,
        ClosurePatchedButNotWalked,
        WholeTreePatchedButNever,
    }

    fn arm_named(name: &str) -> Option<Arm> {
        match name {
            "p1" => Some(Arm::Today),
            "p2a" => Some(Arm::ReleaseChainClosure),
            "p2b16" => Some(Arm::WholeTreeEvery16),
            "p2b256" => Some(Arm::WholeTreeEvery256),
            "whole" => Some(Arm::WholeTreeEveryPublish),
            "single_step" => Some(Arm::OneNodeAtATime),
            "closure_off" => Some(Arm::ClosurePatchedButNotWalked),
            "p2b_never" => Some(Arm::WholeTreePatchedButNever),
            _ => None,
        }
    }

    /// 这个进程的臂：环境变量 `E206_ARM` 在进程开头读一次。
    #[must_use]
    pub fn arm() -> Arm {
        static ARM: OnceLock<Arm> = OnceLock::new();
        *ARM.get_or_init(|| {
            let name = std::env::var("E206_ARM").expect("E206_ARM 要设成一条臂或对照的名字");
            arm_named(&name).unwrap_or_else(|| panic!("E206_ARM 认不得：{name}"))
        })
    }

    /// 这一臂的轮数上界：只有 `single_step` 放到 4096（免得一个一个并的时候先撞上 64，登记第五节阳性对照表）。
    #[must_use]
    pub fn rounds_limit() -> usize {
        if arm() == Arm::OneNodeAtATime {
            4096
        } else {
            ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT
        }
    }

    /// 改起点那一族每第几次发布整棵重写；不是这一族的臂没有。
    #[must_use]
    pub fn whole_tree_every() -> Option<u64> {
        match arm() {
            Arm::WholeTreeEvery16 => Some(16),
            Arm::WholeTreeEvery256 => Some(256),
            Arm::WholeTreeEveryPublish => Some(1),
            Arm::WholeTreePatchedButNever => Some(u64::MAX),
            _ => None,
        }
    }

    /// 这一臂走不走释放链闭包：只有 `p2a` 走，`closure_off` 补丁在、闭包那一步不走。
    #[must_use]
    pub fn closure_is_walked() -> bool {
        matches!(arm(), Arm::ReleaseChainClosure)
    }

    /// 一轮走法（读法第 2 条）。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum RoundExit {
        ReleaseNotCovered,
        PlacementNotCovered,
        Settled,
    }
    impl RoundExit {
        #[must_use]
        pub fn tag(self) -> &'static str {
            match self {
                Self::ReleaseNotCovered => "release_not_covered",
                Self::PlacementNotCovered => "placement_not_covered",
                Self::Settled => "settled",
            }
        }
    }

    /// 进的是哪一份固定点。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Entry {
        Version,
        RowPublish,
    }
'''

MODULE_PART_STATE = r'''
    /// 第 2、3、5、6 组：一轮的读数，只在量窗里记。
    #[derive(Clone, Debug, Default)]
    pub struct RoundReading {
        pub round: u64,
        pub roles_rewritten: u64,
        pub nodes_rewritten: u64,
        pub placements_released: u64,
        pub rewrite_set_size: u64,
        pub nodes_after_size: u64,
        pub release_dirty_new_leaves: u64,
        pub release_dirty_new_internal: u64,
        pub placement_dirty_new_leaves: u64,
        pub placement_dirty_new_internal: u64,
        pub holding_differs: bool,
        pub exit: Option<RoundExit>,
        pub roles_changed_between_rounds: u64,
        pub records_changed_between_rounds: u64,
        /// 三棵码 2 树（规划的次序）这一轮与上一轮 key 集的对称差。
        pub tree_keys_changed_between_rounds: Vec<u64>,
        pub closure_hops: u64,
        pub closure_nodes_added: u64,
        pub closure_lookup_steps: u64,
        /// 这一轮两道之间每行多了多少：(行名, 调用数, 步数, 查询层数之和)。
        pub row_steps: Vec<(&'static str, u64, u64, u64)>,
    }

    /// 一次固定点：第 1、4、6、7 组。
    #[derive(Clone, Debug)]
    pub struct SettleRecord {
        pub settle_call: u64,
        pub entry: Entry,
        /// 这一条固定点属于这次用户改动的第几次判（0 起；与 `e203_scan::take_finished` 交回的下标对得上）。
        pub judgement: u64,
        pub rounds: u64,
        pub outcome: &'static str,
        pub final_roles: u64,
        pub final_nodes: u64,
        pub round_work: u64,
        pub round_work_nodes: u64,
        pub final_set_hash: u64,
        pub seed_nodes: u64,
        pub closure_hops: u64,
        pub closure_nodes_added: u64,
        pub closure_lookup_steps: u64,
        pub slots_placed: u64,
        pub previous_nodes: u64,
        pub previous_records: u64,
        pub active_leaves_by_device: Vec<u64>,
        pub tree_height: u64,
        pub release_chain_depth: u64,
        pub release_chain_nodes: u64,
        pub rounds_detail: Vec<RoundReading>,
    }

    #[derive(Default)]
    struct Thread {
        window: bool,
        settle_calls: u64,
        publishes_in_this_mount: u64,
        open: Option<SettleRecord>,
        current_round: Option<RoundReading>,
        roles_of_the_previous_round: BTreeSet<TransactionUnit>,
        records_of_the_previous_round: Vec<AllocationRecord>,
        records_of_this_round: Vec<AllocationRecord>,
        code_two_keys_of_the_previous_round: Vec<BTreeSet<u64>>,
        code_two_keys_of_this_round: Vec<BTreeSet<u64>>,
        row_totals_at_the_round_begin: Vec<(&'static str, u64, u64, u64)>,
        finished: Vec<SettleRecord>,
    }
    thread_local! {
        static THREAD: RefCell<Thread> = RefCell::new(Thread::default());
    }

    /// 打开或关上量窗：开着时记第 2–6 组与第 7 组的逐轮部分，关着时只记第 1 组（历史那一段）。
    pub fn set_window(open: bool) {
        THREAD.with(|thread_cell| thread_cell.borrow_mut().window = open);
    }
    /// 重挂载之后「同一次挂载里第几次发布」从头数（`p2b` 那一族）。
    pub fn mount_changed() {
        THREAD.with(|thread_cell| thread_cell.borrow_mut().publishes_in_this_mount = 0);
    }
    /// 交回到此刻收好的固定点并清空。
    #[must_use]
    pub fn take_settles() -> Vec<SettleRecord> {
        THREAD.with(|thread_cell| std::mem::take(&mut thread_cell.borrow_mut().finished))
    }

    fn fingerprint(bytes: &[u8]) -> u64 {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash
    }

    fn count(value: usize) -> u64 {
        u64::try_from(value).expect("个数装得进 u64")
    }

    fn is_leaf(node: AllocationRecordTreeNode) -> bool {
        matches!(node, AllocationRecordTreeNode::BelowTheRoot(position) if position.level == 0)
    }

    /// `dirty` 里不在 `covered`、也不在 `also_covered` 里的节点，叶与内部节点分开数（根算内部节点）。
    fn new_dirty(
        dirty: &BTreeSet<AllocationRecordTreeNode>,
        also_covered: Option<&BTreeSet<AllocationRecordTreeNode>>,
        covered: &BTreeSet<AllocationRecordTreeNode>,
    ) -> (u64, u64) {
        let new: Vec<AllocationRecordTreeNode> = dirty
            .iter()
            .filter(|node| !covered.contains(*node))
            .filter(|node| also_covered.is_none_or(|set| !set.contains(*node)))
            .copied()
            .collect();
        let leaves = new.iter().filter(|node| is_leaf(**node)).count();
        (count(leaves), count(new.len() - leaves))
    }

    /// 两份分配记录按 (盘, 槽) 比，不同的条数（一边没有的也算）。
    fn records_differing(before: &[AllocationRecord], after: &[AllocationRecord]) -> u64 {
        let keyed = |records: &[AllocationRecord]| -> BTreeMap<(u32, u64), AllocationRecord> {
            records
                .iter()
                .map(|record| ((record.device.0, record.slot.0), *record))
                .collect()
        };
        let (before, after) = (keyed(before), keyed(after));
        let only_or_different = before
            .iter()
            .filter(|(key, record)| after.get(*key) != Some(*record))
            .count();
        let only_after = after.keys().filter(|key| !before.contains_key(*key)).count();
        count(only_or_different + only_after)
    }

    /// 定下那一轮比上一版新增落点的槽数之和（每盘一条，按 (盘, 槽) 比；旧记录已释放或代变了的算新落点）。
    fn slots_placed(previous: &[AllocationRecord], settled: &[AllocationRecord]) -> u64 {
        let before: HashMap<(u32, u64), AllocationRecord> = previous
            .iter()
            .map(|record| ((record.device.0, record.slot.0), *record))
            .collect();
        settled
            .iter()
            .filter(|record| !record.is_released)
            .filter(|record| {
                before
                    .get(&(record.device.0, record.slot.0))
                    .is_none_or(|old| old.is_released || old.generation != record.generation)
            })
            .map(|record| u64::from(record.span_slots))
            .sum()
    }
'''

MODULE_PART_WALK = r'''
    /// 从 `start` 起按「节点 → 上一版的指针 → 两份落点 → 那片叶与祖先」走（登记第三节 `release_chain` 那一串）：`insert` 交回节点是不是新的。
    /// 交回 (跳数, 线性 `pointer_of` 的步数, 层数)。
    fn walk_the_release_chain(
        start: &BTreeSet<AllocationRecordTreeNode>,
        previous_tree: Option<&AllocationRecordTreeVersion>,
        geometry: &AllocationRecordTreeGeometry,
        mut insert: impl FnMut(AllocationRecordTreeNode) -> bool,
    ) -> (u64, u64, u64) {
        let (mut hops, mut lookup_steps, mut depth) = (0, 0, 0);
        let Some(tree) = previous_tree else {
            return (hops, lookup_steps, depth);
        };
        let mut frontier: Vec<AllocationRecordTreeNode> = start.iter().copied().collect();
        while !frontier.is_empty() {
            depth += 1;
            let mut next = Vec::new();
            for node in frontier {
                hops += 1;
                let position = tree.nodes.iter().position(|(candidate, _)| *candidate == node);
                lookup_steps += count(position.map_or(tree.nodes.len(), |found| found + 1));
                let Some(pointer) = position.map(|found| tree.nodes[found].1) else {
                    continue;
                };
                for location in pointer.locations {
                    let mut climbing = Some(geometry.leaf_of_slot(location.device, location.slot));
                    while let Some(ancestor) = climbing {
                        if insert(ancestor) {
                            next.push(ancestor);
                        }
                        climbing = geometry.parent_of(ancestor);
                    }
                }
            }
            frontier = next;
        }
        (hops, lookup_steps, depth)
    }

    /// 一次固定点开始。
    pub fn settle_begin(
        entry: Entry,
        geometry: &AllocationRecordTreeGeometry,
        previous_nodes: &BTreeSet<AllocationRecordTreeNode>,
        previous_records: &[AllocationRecord],
    ) {
        let judgement = count(super::e203_scan::finished_records());
        THREAD.with(|thread_cell| {
            let thread = &mut *thread_cell.borrow_mut();
            thread.settle_calls += 1;
            if entry == Entry::Version {
                thread.publishes_in_this_mount += 1;
            }
            let (active_leaves_by_device, tree_height) = if thread.window {
                let mut leaves: BTreeMap<u32, u64> = BTreeMap::new();
                for node in nodes_holding_records(geometry, previous_records) {
                    if let AllocationRecordTreeNode::BelowTheRoot(position) = node {
                        if position.level == 0 {
                            *leaves.entry(position.device.0).or_insert(0) += 1;
                        }
                    }
                }
                (leaves.into_values().collect(), geometry.height())
            } else {
                (Vec::new(), 0)
            };
            thread.open = Some(SettleRecord {
                settle_call: thread.settle_calls,
                entry,
                judgement,
                rounds: 0,
                outcome: "open",
                final_roles: 0,
                final_nodes: 0,
                round_work: 0,
                round_work_nodes: 0,
                final_set_hash: 0,
                seed_nodes: 0,
                closure_hops: 0,
                closure_nodes_added: 0,
                closure_lookup_steps: 0,
                slots_placed: 0,
                previous_nodes: count(previous_nodes.len()),
                previous_records: count(previous_records.len()),
                active_leaves_by_device,
                tree_height,
                release_chain_depth: 0,
                release_chain_nodes: 0,
                rounds_detail: Vec::new(),
            });
            thread.current_round = None;
            thread.roles_of_the_previous_round.clear();
            thread.records_of_the_previous_round = if thread.window {
                previous_records.to_vec()
            } else {
                Vec::new()
            };
            thread.records_of_this_round.clear();
            thread.code_two_keys_of_the_previous_round.clear();
            thread.code_two_keys_of_this_round.clear();
            thread.row_totals_at_the_round_begin.clear();
        });
    }

    /// 改起点那一族（P2b、`whole`）：这一次发布该整棵重写时，起点取上一版全部节点（`transaction.rs:13375`）。
    pub fn seed_the_rewrite_set(
        rewritten: &mut BTreeSet<AllocationRecordTreeNode>,
        previous_nodes: &BTreeSet<AllocationRecordTreeNode>,
    ) {
        let Some(every) = whole_tree_every() else {
            return;
        };
        THREAD.with(|thread_cell| {
            let thread = &mut *thread_cell.borrow_mut();
            if thread.publishes_in_this_mount % every == 0 {
                rewritten.extend(previous_nodes.iter().copied());
                if let Some(open) = thread.open.as_mut() {
                    open.seed_nodes = count(previous_nodes.len());
                }
            }
        });
    }
'''

MODULE_PART_ROUNDS = r'''
    /// 收上一轮：记进这次固定点的逐轮读数，算它与再上一轮的差。
    fn close_the_round(thread: &mut Thread) {
        let Some(mut round) = thread.current_round.take() else {
            return;
        };
        let totals = super::e203_scan::between_row_totals();
        let before = &thread.row_totals_at_the_round_begin;
        round.row_steps = totals
            .iter()
            .filter_map(|(row, calls, steps, query_levels)| {
                let (old_calls, old_steps, old_levels) = before
                    .iter()
                    .find(|(name, _, _, _)| name == row)
                    .map_or((0, 0, 0), |(_, calls_then, steps_then, levels_then)| (*calls_then, *steps_then, *levels_then));
                (*calls > old_calls || *steps > old_steps || *query_levels > old_levels)
                    .then_some((*row, *calls - old_calls, *steps - old_steps, *query_levels - old_levels))
            })
            .collect();
        thread.row_totals_at_the_round_begin = totals;
        round.records_changed_between_rounds =
            records_differing(&thread.records_of_the_previous_round, &thread.records_of_this_round);
        thread.records_of_the_previous_round = std::mem::take(&mut thread.records_of_this_round);
        let trees = thread
            .code_two_keys_of_this_round
            .len()
            .max(thread.code_two_keys_of_the_previous_round.len());
        let empty = BTreeSet::new();
        round.tree_keys_changed_between_rounds = (0..trees)
            .map(|tree| {
                let now = thread.code_two_keys_of_this_round.get(tree).unwrap_or(&empty);
                let then = thread.code_two_keys_of_the_previous_round.get(tree).unwrap_or(&empty);
                count(now.symmetric_difference(then).count())
            })
            .collect();
        thread.code_two_keys_of_the_previous_round = std::mem::take(&mut thread.code_two_keys_of_this_round);
        if let Some(open) = thread.open.as_mut() {
            open.rounds_detail.push(round);
        }
    }

    /// 第 r 轮开头（`transaction.rs:13379` 之前）。
    pub fn round_begin(rewrite_set_size: usize, nodes_after_size: usize) {
        THREAD.with(|thread_cell| {
            let thread = &mut *thread_cell.borrow_mut();
            close_the_round(thread);
            let Some(open) = thread.open.as_mut() else {
                return;
            };
            open.rounds += 1;
            if thread.window {
                thread.current_round = Some(RoundReading {
                    round: open.rounds,
                    rewrite_set_size: count(rewrite_set_size),
                    nodes_after_size: count(nodes_after_size),
                    ..RoundReading::default()
                });
            }
        });
    }

    /// 规划之后这一轮的全部角色与分配记录树节点（`transaction.rs:13388`、`:13386`）。
    pub fn round_resolved(roles: &[TransactionUnit], nodes_rewritten: usize) {
        THREAD.with(|thread_cell| {
            let thread = &mut *thread_cell.borrow_mut();
            let Some(open) = thread.open.as_mut() else {
                return;
            };
            open.round_work += count(roles.len());
            open.round_work_nodes += count(nodes_rewritten);
            open.final_nodes = count(nodes_rewritten);
            if let Some(round) = thread.current_round.as_mut() {
                round.roles_rewritten = count(roles.len());
                round.nodes_rewritten = count(nodes_rewritten);
                let now: BTreeSet<TransactionUnit> = roles.iter().copied().collect();
                round.roles_changed_between_rounds =
                    count(now.symmetric_difference(&thread.roles_of_the_previous_round).count());
                thread.roles_of_the_previous_round = now;
            }
        });
    }

    /// 这一轮列出的释放（`transaction.rs:13421` 那个 `for` 之后）。
    pub fn round_released(placements: usize) {
        THREAD.with(|thread_cell| {
            if let Some(round) = thread_cell.borrow_mut().current_round.as_mut() {
                round.placements_released = count(placements);
            }
        });
    }

    /// 走完释放（`transaction.rs:13431` 之后）：释放弄脏、计划还没罩住的节点；第一轮另走释放链，报层数与闭包大小（Q10）。
    pub fn round_after_the_release(
        geometry: &AllocationRecordTreeGeometry,
        changed_by_the_release: &BTreeSet<AllocationRecordTreeNode>,
        rewritten_by_the_plan: &BTreeSet<AllocationRecordTreeNode>,
        records: &[AllocationRecord],
        previous_tree: Option<&AllocationRecordTreeVersion>,
    ) {
        THREAD.with(|thread_cell| {
            let thread = &mut *thread_cell.borrow_mut();
            let Some(round) = thread.current_round.as_mut() else {
                return;
            };
            let (leaves, internal) = new_dirty(changed_by_the_release, None, rewritten_by_the_plan);
            round.release_dirty_new_leaves = leaves;
            round.release_dirty_new_internal = internal;
            thread.records_of_this_round = records.to_vec();
            if round.round == 1 {
                let mut reached = changed_by_the_release.clone();
                let (_, _, depth) = walk_the_release_chain(changed_by_the_release, previous_tree, geometry, |node| reached.insert(node));
                if let Some(open) = thread.open.as_mut() {
                    open.release_chain_depth = depth;
                    open.release_chain_nodes = count(reached.len());
                }
            }
        });
    }

    /// 走完取落点（`transaction.rs:13444`、`:13446` 之后）。
    pub fn round_after_the_placements(
        changed: &BTreeSet<AllocationRecordTreeNode>,
        changed_by_the_release: &BTreeSet<AllocationRecordTreeNode>,
        rewritten_by_the_plan: &BTreeSet<AllocationRecordTreeNode>,
        holding_records: &BTreeSet<AllocationRecordTreeNode>,
        nodes_after: &BTreeSet<AllocationRecordTreeNode>,
        records: &[AllocationRecord],
    ) {
        THREAD.with(|thread_cell| {
            let thread = &mut *thread_cell.borrow_mut();
            let Some(round) = thread.current_round.as_mut() else {
                return;
            };
            let (leaves, internal) = new_dirty(changed, Some(changed_by_the_release), rewritten_by_the_plan);
            round.placement_dirty_new_leaves = leaves;
            round.placement_dirty_new_internal = internal;
            round.holding_differs = holding_records != nodes_after;
            thread.records_of_this_round = records.to_vec();
        });
    }

    /// 并进重写集（`transaction.rs:13434`、`:13455`）。今天的做法整批并；`single_step` 只并差集里按序最小的一个；`p2a` 并完接着走释放链闭包。
    pub fn merge_into_the_rewrite_set(
        rewritten: &mut BTreeSet<AllocationRecordTreeNode>,
        dirty: BTreeSet<AllocationRecordTreeNode>,
        rewritten_by_the_plan: &BTreeSet<AllocationRecordTreeNode>,
        previous_tree: Option<&AllocationRecordTreeVersion>,
        geometry: &AllocationRecordTreeGeometry,
        exit: RoundExit,
    ) {
        if arm() == Arm::OneNodeAtATime {
            if let Some(smallest) = dirty.iter().find(|node| !rewritten_by_the_plan.contains(*node)) {
                rewritten.insert(*smallest);
            }
        } else {
            rewritten.extend(dirty.iter().copied());
        }
        let (mut hops, mut added, mut lookup_steps) = (0, 0, 0);
        if closure_is_walked() {
            (hops, lookup_steps, _) = walk_the_release_chain(&dirty, previous_tree, geometry, |node| {
                let new = rewritten.insert(node);
                added += u64::from(new);
                new
            });
        }
        THREAD.with(|thread_cell| {
            let thread = &mut *thread_cell.borrow_mut();
            if let Some(open) = thread.open.as_mut() {
                open.closure_hops += hops;
                open.closure_nodes_added += added;
                open.closure_lookup_steps += lookup_steps;
            }
            if let Some(round) = thread.current_round.as_mut() {
                round.exit = Some(exit);
                round.closure_hops = hops;
                round.closure_nodes_added = added;
                round.closure_lookup_steps = lookup_steps;
            }
        });
    }

    /// 定下（`transaction.rs:13448`）：第 7 组的哈希与落点槽数。
    pub fn round_settled(
        roles: &[TransactionUnit],
        release: &[Placement],
        records: &[AllocationRecord],
        previous_records: &[AllocationRecord],
    ) {
        let hash = fingerprint(format!("{roles:?}|{release:?}").as_bytes());
        let placed = slots_placed(previous_records, records);
        THREAD.with(|thread_cell| {
            let thread = &mut *thread_cell.borrow_mut();
            if let Some(open) = thread.open.as_mut() {
                open.final_roles = count(roles.len());
                open.final_set_hash = hash;
                open.slots_placed = placed;
            }
            if let Some(round) = thread.current_round.as_mut() {
                round.exit = Some(RoundExit::Settled);
            }
        });
    }

    /// 这一轮规划里一棵多层码 2 树进出的 key（`code_two_tree.rs` 里 E203 记 `code_two_keys_changed` 的同一处；只记归口到三棵树那一次规划的）。
    pub fn note_code_two_keys<'keys, Key: std::fmt::Debug + 'keys>(keys: impl Iterator<Item = &'keys Key>) {
        if !THREAD.with(|thread_cell| thread_cell.borrow().window) || !super::e203_scan::attributed_to_multi_level_planning() {
            return;
        }
        let set: BTreeSet<u64> = keys.map(|key| fingerprint(format!("{key:?}").as_bytes())).collect();
        THREAD.with(|thread_cell| thread_cell.borrow_mut().code_two_keys_of_this_round.push(set));
    }

    /// 固定点交回（调用处 `transaction.rs:12861`、`:2099` 的 `?` 之前）：定结局、收这一条。
    pub fn settle_outcome<Settled>(outcome: &Result<Settled, PublishError>) {
        let tag = match outcome {
            Ok(_) => "settled",
            Err(error) => {
                let text = format!("{error:?}");
                if text.starts_with("PlacementRefused") {
                    "placement_refused"
                } else if text.starts_with("AllocationRecordTreeRewriteSetDidNotSettle") {
                    "did_not_settle"
                } else {
                    "other_error"
                }
            }
        };
        THREAD.with(|thread_cell| {
            let thread = &mut *thread_cell.borrow_mut();
            close_the_round(thread);
            if let Some(mut open) = thread.open.take() {
                open.outcome = tag;
                thread.finished.push(open);
            }
        });
    }
}
'''


class Edit(e203.Edit):
    """一处 E206 的改动（字段同 E203 脚本的 Edit）；只有一种构建插，timing 那一格不用。"""

    def __init__(self, ident, file, head, end, anchor, mode, inserted):
        super().__init__(ident, file, head, end, anchor, mode, inserted, None)


TREE = "previous_tree"
# 实现的并重写集是 `merge_into_the_rewrite_set_walking_the_release_chain`（并进之后顺着释放链走到闭包，E206 的 P2a 臂就是它）；
# 两处调用整句换成 E206 自己的并法，按臂决定走不走闭包。
RELEASE_MERGE = ("            merge_into_the_rewrite_set_walking_the_release_chain(\n                &mut rewritten,\n                changed_by_the_release,\n"
                 "                previous_tree,\n                &geometry,\n            );\n")
PLACEMENT_MERGE = ("        merge_into_the_rewrite_set_walking_the_release_chain(\n            &mut rewritten,\n            changed,\n"
                   "            previous_tree,\n            &geometry,\n        );\n")
ROUND_BEGIN_ANCHOR = "        let rewritten_among_the_nodes_after: BTreeSet<AllocationRecordTreeNode> =\n"
NODES_AFTER_DECLARED = "    let mut nodes_after: BTreeSet<AllocationRecordTreeNode> = previous_nodes.clone();\n"
LIMIT_LOOP = "    for round_index in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {\n"
REHEARSAL = "        let mut rehearsal = allocator.clone();\n"
CHANGED_BY_THE_RELEASE = "        let changed_by_the_release =\n            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());\n"
HOLDING = "        let holding_records = nodes_holding_records(&geometry, rehearsal.records());\n"
MERGE_ARGUMENTS = "&rewritten_by_the_plan, " + TREE + ", &geometry, "


def merged(dirty, exit_name):
    return (f"        {E}merge_into_the_rewrite_set(&mut rewritten, {dirty}, {MERGE_ARGUMENTS}{E}RoundExit::{exit_name});\n")


def edits():
    """第三节「每个读数的取样位置」逐个读数的插入点；两条臂的补丁都经 `merge_into_the_rewrite_set`、`seed_the_rewrite_set`、`rounds_limit`。"""
    return [
        Edit("call_site_open", T, HEAD_JUDGE, FN_END, "    let settled = settle_the_allocation_record_tree(\n", "replace",
             "    let e206_settling_outcome = settle_the_allocation_record_tree(\n"),
        Edit("call_site_close", T, HEAD_JUDGE, FN_END, "        pool.snapshot_lifetime_judgement(),\n    )?;\n", "replace",
             f"        pool.snapshot_lifetime_judgement(),\n    );\n    {E}settle_outcome(&e206_settling_outcome);\n    let settled = e206_settling_outcome?;\n"),
        Edit("settle_begin", T, HEAD_SETTLE, FN_END, NODES_AFTER_DECLARED, "after",
             f"    {E}settle_begin({E}Entry::Version, &geometry, &previous_nodes, previous_records);\n"
             f"    {E}seed_the_rewrite_set(&mut rewritten, &previous_nodes);\n"),
        Edit("rounds_limit", T, HEAD_SETTLE, FN_END, LIMIT_LOOP, "replace", f"    for round_index in 0..{E}rounds_limit() {{\n"),
        Edit("round_begin", T, HEAD_SETTLE, FN_END, ROUND_BEGIN_ANCHOR, "before", f"        {E}round_begin(rewritten.len(), nodes_after.len());\n"),
        Edit("round_resolved", T, HEAD_SETTLE, FN_END, "        let (previous_roles_replaced, mut release) =\n", "before",
             f"        {E}round_resolved(&resolved.rewritten_roles, rewritten_by_the_plan.len());\n"),
        Edit("round_released", T, HEAD_SETTLE, FN_END, REHEARSAL, "before", f"        {E}round_released(release.len());\n"),
        Edit("round_after_the_release", T, HEAD_SETTLE, FN_END, CHANGED_BY_THE_RELEASE, "after",
             f"        {E}round_after_the_release(&geometry, &changed_by_the_release, &rewritten_by_the_plan, rehearsal.records(), {TREE});\n"),
        Edit("merge_after_the_release", T, HEAD_SETTLE, FN_END, RELEASE_MERGE, "replace",
             "    " + merged("changed_by_the_release", "ReleaseNotCovered")),
        Edit("round_after_the_placements", T, HEAD_SETTLE, FN_END, HOLDING, "after",
             f"        {E}round_after_the_placements(&changed, &changed_by_the_release, &rewritten_by_the_plan, &holding_records, &nodes_after, rehearsal.records());\n"),
        Edit("round_settled", T, HEAD_SETTLE, FN_END, "            return Ok(SettledPublish {\n", "before",
             f"            {E}round_settled(&resolved.rewritten_roles, &release, rehearsal.records(), previous_records);\n"),
        Edit("merge_after_the_placements", T, HEAD_SETTLE, FN_END, PLACEMENT_MERGE, "replace", merged("changed", "PlacementNotCovered")),
        # 另一份固定点（树表 0 条那一版写行）：只记第 1 组，不带臂
        Edit("row_call_site_open", T, HEAD_ROW_CALLER, FN_END,
             "    let settled = settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file(\n", "replace",
             "    let e206_settling_outcome = settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file(\n"),
        Edit("row_call_site_close", T, HEAD_ROW_CALLER, FN_END, "        plan.instance_table,\n        txg,\n        allocator,\n    )?;\n", "replace",
             f"        plan.instance_table,\n        txg,\n        allocator,\n    );\n    {E}settle_outcome(&e206_settling_outcome);\n    let settled = e206_settling_outcome?;\n"),
        Edit("row_settle_begin", T, HEAD_ROW_SETTLE, FN_END, NODES_AFTER_DECLARED, "after",
             f"    {E}settle_begin({E}Entry::RowPublish, &geometry, &previous_nodes, previous_records);\n"),
        Edit("row_round_begin", T, HEAD_ROW_SETTLE, FN_END, ROUND_BEGIN_ANCHOR, "before", f"        {E}round_begin(rewritten.len(), nodes_after.len());\n"),
        Edit("row_round_resolved", T, HEAD_ROW_SETTLE, FN_END, REHEARSAL, "before", f"        {E}round_resolved(&rewritten_roles, rewritten_by_the_plan.len());\n"),
        Edit("row_round_settled", T, HEAD_ROW_SETTLE, FN_END, "            return Ok(SettledRowPublishOnAVersionWithoutFile {\n", "before",
             f"            {E}round_settled(&rewritten_roles, &release, rehearsal.records(), previous_records);\n"),
        # 三棵多层码 2 树每轮进出的 key：与 E203 的 code_two_keys_changed 同一处
        Edit("code_two_keys", CODE2, HEAD_CODE_TWO_PLAN, FN_END, "    Ok(CodeTwoTreePlan {\n        shape: CodeTwoTreeShape { nodes },\n", "before",
             f"    {E}note_code_two_keys(previous_keys.symmetric_difference(&keys_after_this_publish));\n"),
        Edit("code_two_keys_zero_root", CODE2, HEAD_ZERO_ROOT, FN_END,
             "    if !keys_after_this_publish.is_empty() {\n        return plan_the_tree_after_this_publish(previous, keys_after_this_publish, capacity);\n    }\n", "after",
             f"    {E}note_code_two_keys(previous.keys_in_order().into_iter());\n"),
    ]


def module_text():
    return MODULE_PART_ARMS + MODULE_PART_STATE + MODULE_PART_WALK + MODULE_PART_ROUNDS


READINGS_PLAIN = r'''//! E206（发布固定点轮数的代价）`--build plain` 的读数：实现一个字节没插，全空。历史用例只比 `image_sha256`（作废条款第 5 条）。
#![allow(dead_code, reason = "历史用例在这一份构建里只用到一部分")]

use super::Pool;

pub fn build_name() -> &'static str {
    "plain"
}
pub fn set_window(_open: bool) {}
pub fn mount_changed() {}
pub fn pause_readings(_paused: bool) {}
pub fn take_settle_lines(_item: &str, _labels: &str, _with_rounds: bool, _history: &mut HistoryLines) -> Vec<String> {
    Vec::new()
}
pub fn take_record_lines(_item: &str, _labels: &str, _publish_prefix: &str) -> Vec<String> {
    Vec::new()
}
pub fn state_line(_item: &str, _labels: &str, _pool: &Pool) -> String {
    String::new()
}
pub fn sizes_line(_item: &str) -> String {
    String::new()
}
pub fn constants_line(_item: &str, _pool: &Pool) -> String {
    String::new()
}
pub fn slots_placed_since_taken() -> u64 {
    0
}
pub fn settle_calls_taken() -> u64 {
    0
}

#[derive(Default)]
pub struct HistoryLines {
    pub peak_rounds: u64,
    pub peak_round_work: u64,
    pub settle_calls: u64,
}
impl HistoryLines {
    pub fn close_the_bucket(&mut self, _item: &str) -> Option<String> {
        None
    }
}
'''

READINGS_COUNTS = r'''//! E206（发布固定点轮数的代价）`--build counts` 的读数行：第三节每组读数打成 `E206RESULT` 行，模型 `e206_fixed_point_cost.rs` 读它们。
//! E203（准入之前扫类的步代价）的逐步计数（`row`、`delta`、`publish`、`state`）照 E203 的字段打，只把定位字段换成 `publish=<状态>/<操作>/<第几次>/<第几次判>`。
#![allow(dead_code, reason = "历史用例只用到一部分")]

use super::Pool;
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::code_two_tree::CodeTwoTreeKey;
use singlefs_core::records::AccountingEntry;
use singlefs_core::transaction::e203_scan::{self, PublishRecord};
use singlefs_core::transaction::e206_fixed_point::{self, Entry, RoundReading, SettleRecord};
use singlefs_core::transaction::{
    MultiLevelCodeTwoTree, TransactionUnit, ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT,
};
use singlefs_format::{ALLOCATION_RECORD_TREE_INTERNAL_FANOUT, ALLOCATION_RECORD_TREE_LEAF_SLOTS};
use std::cell::Cell;
use std::fmt::Write as _;

/// 历史那一段每 1000 次固定点收一行直方图（历史第 2 条）。
const HISTORY_BUCKET_CALLS: u64 = 1000;

thread_local! {
    static SLOTS_PLACED: Cell<u64> = const { Cell::new(0) };
    static SETTLE_CALLS_TAKEN: Cell<u64> = const { Cell::new(0) };
}

pub fn build_name() -> &'static str {
    "counts"
}
pub fn set_window(open: bool) {
    e206_fixed_point::set_window(open);
}
pub fn mount_changed() {
    e206_fixed_point::mount_changed();
}
pub fn pause_readings(paused: bool) {
    e203_scan::pause_readings(paused);
}
/// 上一次问过之后定下的固定点新增落点的槽数之和（第七节 ③ 对账）。
pub fn slots_placed_since_taken() -> u64 {
    SLOTS_PLACED.with(|cell| cell.replace(0))
}
/// 到此刻取走过多少次固定点（阴性对照判「走到固定点了没有」）。
pub fn settle_calls_taken() -> u64 {
    SETTLE_CALLS_TAKEN.with(Cell::get)
}

/// 历史那一段的直方图：一桶里每次固定点的 轮数:角色数:节点数:轮功:哈希:落点槽数:结局 连成一串，模型逐个比（Q5、阳性对照）。
#[derive(Default)]
pub struct HistoryLines {
    bucket: u64,
    calls: u64,
    rounds_one: u64,
    rounds_two: u64,
    rounds_three_or_more: u64,
    peak_rounds_in_the_bucket: u64,
    peak_round_work_in_the_bucket: u64,
    peak_final_roles_in_the_bucket: u64,
    sequence: String,
    pub peak_rounds: u64,
    pub peak_round_work: u64,
    pub settle_calls: u64,
}

fn outcome_code(outcome: &str) -> char {
    match outcome {
        "settled" => 's',
        "placement_refused" => 'r',
        "did_not_settle" => 'd',
        _ => 'e',
    }
}

impl HistoryLines {
    fn add(&mut self, record: &SettleRecord) {
        self.calls += 1;
        self.settle_calls += 1;
        match record.rounds {
            0 | 1 => self.rounds_one += 1,
            2 => self.rounds_two += 1,
            _ => self.rounds_three_or_more += 1,
        }
        self.peak_rounds_in_the_bucket = self.peak_rounds_in_the_bucket.max(record.rounds);
        self.peak_round_work_in_the_bucket = self.peak_round_work_in_the_bucket.max(record.round_work);
        self.peak_final_roles_in_the_bucket = self.peak_final_roles_in_the_bucket.max(record.final_roles);
        self.peak_rounds = self.peak_rounds.max(record.rounds);
        self.peak_round_work = self.peak_round_work.max(record.round_work);
        let _ = write!(
            self.sequence,
            "{}{}:{}:{}:{}:{:016x}:{}:{}",
            if self.sequence.is_empty() { "" } else { ";" },
            record.rounds,
            record.final_roles,
            record.final_nodes,
            record.round_work,
            record.final_set_hash,
            record.slots_placed,
            outcome_code(record.outcome)
        );
    }

    /// 收这一桶；一次都没有的桶没有行。
    pub fn close_the_bucket(&mut self, item: &str) -> Option<String> {
        if self.calls == 0 {
            return None;
        }
        let line = format!(
            "E206RESULT name=history_histogram item={item} bucket={} calls={} rounds_one={} rounds_two={} rounds_three_or_more={} peak_rounds={} peak_round_work={} peak_final_roles={} sequence={}",
            self.bucket,
            self.calls,
            self.rounds_one,
            self.rounds_two,
            self.rounds_three_or_more,
            self.peak_rounds_in_the_bucket,
            self.peak_round_work_in_the_bucket,
            self.peak_final_roles_in_the_bucket,
            self.sequence
        );
        self.bucket += 1;
        self.calls = 0;
        self.rounds_one = 0;
        self.rounds_two = 0;
        self.rounds_three_or_more = 0;
        self.peak_rounds_in_the_bucket = 0;
        self.peak_round_work_in_the_bucket = 0;
        self.peak_final_roles_in_the_bucket = 0;
        self.sequence.clear();
        Some(line)
    }
}

fn joined(values: &[u64]) -> String {
    let texts: Vec<String> = values.iter().map(u64::to_string).collect();
    if texts.is_empty() { "-".to_string() } else { texts.join(",") }
}

fn settle_line(name: &str, item: &str, labels: &str, record: &SettleRecord) -> String {
    format!(
        "E206RESULT name={name} item={item} {labels} settle_call={} entry={} judgement={} rounds={} outcome={} final_roles={} final_nodes={} round_work={} round_work_nodes={} final_set_hash={:016x} seed_nodes={} closure_hops={} closure_nodes_added={} closure_lookup_steps={} slots_placed={} previous_nodes={} previous_records={} active_leaves_by_device={} tree_height={} release_chain_depth={} release_chain_nodes={}",
        record.settle_call,
        if record.entry == Entry::Version { "version" } else { "row_publish" },
        record.judgement,
        record.rounds,
        record.outcome,
        record.final_roles,
        record.final_nodes,
        record.round_work,
        record.round_work_nodes,
        record.final_set_hash,
        record.seed_nodes,
        record.closure_hops,
        record.closure_nodes_added,
        record.closure_lookup_steps,
        record.slots_placed,
        record.previous_nodes,
        record.previous_records,
        joined(&record.active_leaves_by_device),
        record.tree_height,
        record.release_chain_depth,
        record.release_chain_nodes
    )
}

fn round_line(item: &str, labels: &str, settle_call: u64, round: &RoundReading) -> String {
    format!(
        "E206RESULT name=round item={item} {labels} settle_call={settle_call} round={} roles_rewritten={} nodes_rewritten={} placements_released={} rewrite_set_size={} nodes_after_size={} release_dirty_new_leaves={} release_dirty_new_internal={} placement_dirty_new_leaves={} placement_dirty_new_internal={} holding_differs={} round_exit={} roles_changed_between_rounds={} records_changed_between_rounds={} tree_keys_changed_between_rounds={} closure_hops={} closure_nodes_added={} closure_lookup_steps={}",
        round.round,
        round.roles_rewritten,
        round.nodes_rewritten,
        round.placements_released,
        round.rewrite_set_size,
        round.nodes_after_size,
        round.release_dirty_new_leaves,
        round.release_dirty_new_internal,
        round.placement_dirty_new_leaves,
        round.placement_dirty_new_internal,
        u8::from(round.holding_differs),
        round.exit.map_or("not_reached", |exit| exit.tag()),
        round.roles_changed_between_rounds,
        round.records_changed_between_rounds,
        joined(&round.tree_keys_changed_between_rounds),
        round.closure_hops,
        round.closure_nodes_added,
        round.closure_lookup_steps
    )
}
'''

READINGS_COUNTS_TAIL = r'''
/// 第七节 ① 第 3、4 条的用例断言：不符走第十节（F1 的后半句），不作废，所以打一行、不 panic。
/// 每一轮重写集不小于上一轮（`transaction.rs:13377`）；定下的那一轮没有计划之外弄脏的节点、装着记录的节点与这一版的节点相同（`:13447`）。
fn clause_violation_lines(item: &str, labels: &str, record: &SettleRecord) -> Vec<String> {
    let mut lines = Vec::new();
    let mut size_before = 0;
    for round in &record.rounds_detail {
        if round.rewrite_set_size < size_before {
            lines.push(format!(
                "E206RESULT name=clause_violation item={item} {labels} settle_call={} round={} clause=rewrite_set_only_grows size={} size_before={size_before}",
                record.settle_call, round.round, round.rewrite_set_size
            ));
        }
        size_before = round.rewrite_set_size;
        let dirty = round.release_dirty_new_leaves + round.release_dirty_new_internal + round.placement_dirty_new_leaves + round.placement_dirty_new_internal;
        if round.exit.is_some_and(|exit| exit.tag() == "settled") && (dirty != 0 || round.holding_differs) {
            lines.push(format!(
                "E206RESULT name=clause_violation item={item} {labels} settle_call={} round={} clause=settled_round_is_covered dirty={dirty} holding_differs={}",
                record.settle_call, round.round, u8::from(round.holding_differs)
            ));
        }
    }
    lines
}

/// 取走这一次用户改动里的固定点。`with_rounds` 为真是量窗：每条打 `settle`、每轮打 `round` 与它的 `round_row`；
/// 为假是历史：`rounds ≥ 3` 的打 `history_settle`，全部进直方图，满 1000 次收一行。
pub fn take_settle_lines(item: &str, labels: &str, with_rounds: bool, history: &mut HistoryLines) -> Vec<String> {
    let mut lines = Vec::new();
    for record in e206_fixed_point::take_settles() {
        SETTLE_CALLS_TAKEN.with(|cell| cell.set(cell.get() + 1));
        if record.outcome == "settled" {
            SLOTS_PLACED.with(|cell| cell.set(cell.get() + record.slots_placed));
        }
        if with_rounds {
            lines.push(settle_line("settle", item, labels, &record));
            lines.extend(clause_violation_lines(item, labels, &record));
            for round in &record.rounds_detail {
                lines.push(round_line(item, labels, record.settle_call, round));
                for (row, calls, steps, query_levels) in &round.row_steps {
                    lines.push(format!(
                        "E206RESULT name=round_row item={item} {labels} settle_call={} round={} row={row} calls={calls} steps={steps} query_levels={query_levels}",
                        record.settle_call, round.round
                    ));
                }
            }
        } else {
            if record.rounds >= 3 {
                lines.push(settle_line("history_settle", item, labels, &record));
            }
            history.add(&record);
            if history.calls >= HISTORY_BUCKET_CALLS {
                lines.extend(history.close_the_bucket(item));
            }
        }
    }
    lines
}

fn record_lines(item: &str, labels: &str, publish: &str, record: &PublishRecord) -> Vec<String> {
    let mut lines = Vec::new();
    for row in &record.rows {
        lines.push(format!(
            "E206RESULT name=row item={item} {labels} publish={publish} segment={} row={} calls={} steps={} scale={} steps_to_the_hit={} scale_to_the_hit={} queries={} query_levels={}",
            row.segment, row.row, row.calls, row.steps, row.scale, row.steps_to_the_hit, row.scale_to_the_hit, row.queries, row.query_levels
        ));
    }
    for delta in &record.deltas {
        lines.push(format!(
            "E206RESULT name=delta item={item} {labels} publish={publish} segment={} fixed_point_round={} key={} value={}",
            delta.segment, delta.round, delta.key, delta.value
        ));
    }
    lines.push(format!(
        "E206RESULT name=publish item={item} {labels} publish={publish} had_caller_segment={} fixed_point_rounds={}",
        record.had_caller_segment, record.rounds
    ));
    lines
}

/// 取走这一次用户改动里每一次判的 E203 读数；`publish` 字段是 `<标签里的状态>/<操作>/<第几次>/<第几次判>`。
pub fn take_record_lines(item: &str, labels: &str, publish_prefix: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for (judgement, record) in e203_scan::take_finished().iter().enumerate() {
        lines.extend(record_lines(item, labels, &format!("{publish_prefix}/{judgement}"), record));
    }
    lines
}

/// 这一刻的规模读数（第 8 组；同 E203 的 `state_text`）：被量的改动做之前取。
pub fn state_line(item: &str, labels: &str, pool: &Pool) -> String {
    let version = pool.current_file_version();
    let allocator = &pool.session().allocator;
    let heights = version.position_addressed_tree_heights_read_from_the_root_node_headers();
    format!(
        "E206RESULT name=state item={item} {labels} units={} mapped_units={} allocation_records={} cluster_segments={} other_files={} tombstone_containers={} inode_leaf_containers={} accounting_rows={} devices={} whole_copy_elements={} deadlist_entries={} extent_upper_nodes={} extent_upper_levels={} accounting_tree_levels={} deadlist_tree_levels={} allocation_tree_levels={} central_mapping_levels={} frozen_publish_set={}",
        version.units.len(),
        version.mapped_units.len(),
        allocator.records().len(),
        allocator.e203_cluster_segments_opened(),
        version.files_other_than_the_first.len(),
        version.tombstone_containers.len(),
        version.inode_leaf_containers.len(),
        version.accounting_entries.len(),
        allocator.devices.len(),
        allocator.e203_whole_copy_elements(),
        version.deadlist_entries.len(),
        version.extent_tree.upper_nodes.len(),
        heights.extent_tree_upper_segment,
        version.height_read_from_the_root_node_header(MultiLevelCodeTwoTree::Accounting),
        if version.deadlist_entries.is_empty() {
            0
        } else {
            version.height_read_from_the_root_node_header(MultiLevelCodeTwoTree::Deadlist)
        },
        heights.allocation_record_tree,
        version.height_read_from_the_root_node_header(MultiLevelCodeTwoTree::CentralMapping),
        // 发布落盘失败冻结着等重发时是 1：E207 模型把那一点整点作废（E203 的 `state_text` 同一个字段）
        u8::from(allocator.frozen_publish().is_some()),
    )
}

/// 类型大小（第五节 P3 内存式子吃 σ_unit）。
pub fn sizes_line(item: &str) -> String {
    format!(
        "E206RESULT name=sizes item={item} unit_bytes={} key_bytes={} accounting_row_bytes={} allocation_record_bytes={}",
        std::mem::size_of::<TransactionUnit>(),
        std::mem::size_of::<CodeTwoTreeKey>(),
        std::mem::size_of::<AccountingEntry>(),
        std::mem::size_of::<AllocationRecord>()
    )
}

/// 产物头：实现的值，模型本地常量逐个比（第七节 ① 第 1 条、② 第 4 条）。
pub fn constants_line(item: &str, pool: &Pool) -> String {
    format!(
        "E206RESULT name=constants item={item} leaf_slots={ALLOCATION_RECORD_TREE_LEAF_SLOTS} internal_fanout={ALLOCATION_RECORD_TREE_INTERNAL_FANOUT} settling_rounds_at_most={ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT} disks={}",
        pool.session().allocator.devices.len()
    )
}
'''


def readings_text(kind):
    return READINGS_COUNTS + READINGS_COUNTS_TAIL if kind == "counts" else READINGS_PLAIN


class AnchorError(e203.AnchorError):
    pass


def needed_paths():
    return sorted(set(e203.needed_paths()) | {edit.file for edit in edits()})


def transform(files, kind):
    """files: {相对路径: 原文}。plain 原样；counts 先走 E203 的 transform，再插 E206，有一个锚点不是恰好一次就抛 AnchorError、不写。"""
    if kind == "plain":
        return dict(files)
    result = e203.transform(files, "counts", e203.edits())
    for edit in edits():
        if edit.file not in result:
            raise AnchorError(f"文件不在：{edit.file}")
        e203.locate(result[edit.file], edit)
    for edit in edits():
        result[edit.file] = e203.apply_one(result[edit.file], edit, "counts")
    if result[T].count(E203_MODULE_ANCHOR) != 1:
        raise AnchorError(f"E203 模块里的取数锚点命中 {result[T].count(E203_MODULE_ANCHOR)} 次，不是一次")
    result[T] = result[T].replace(E203_MODULE_ANCHOR, E203_MODULE_ACCESSORS + E203_MODULE_ANCHOR) + module_text()
    return result


def build(args):
    kind, source, into = args["--build"], args["--source"], args["--into"]
    if kind not in BUILDS:
        print("--build 只认 counts 或 plain", file=sys.stderr)
        return 2
    files = e203.read_tree(source, needed_paths())
    try:
        changed = transform(files, kind)
    except e203.AnchorError as error:
        print(f"锚点不是恰好命中一次，一个字节没写：{error}", file=sys.stderr)
        return 3
    diff_text = e203.unified(files, changed)
    readings = readings_text(kind)
    try:
        e203.check_source_is_complete(source)
        key = e203.copy_key(source, kind, diff_text + readings + pathlib.Path(__file__).read_text(encoding="utf-8"))
        state = e203.prepare_copy(source, into, key)
        if state == "fresh":
            e203.copy_tree(source, into)
            for path, text in changed.items():
                if text != files[path]:
                    e203.write_atomically(pathlib.Path(into) / path, text)
            target = pathlib.Path(into) / READINGS_IN_THE_COPY
            target.parent.mkdir(parents=True, exist_ok=True)
            e203.write_atomically(target, readings)
    except e203.CopyError as error:
        print(f"副本拷不全：{error}", file=sys.stderr)
        return 2
    if "--diff-out" in args:
        e203.write_atomically(args["--diff-out"], diff_text)
    if state == "fresh":
        e203.mark_copy_done(into, key)
    edit_count = len(e203.edits()) + len(edits()) if kind == "counts" else 0
    print(f"name=counters_applied build={kind} edits={edit_count} files={sum(1 for path in changed if changed[path] != files[path])} diff_lines={diff_text.count(chr(10))} copy={state}")
    return 0


def syntax_problems(files, label):
    """不起编译的判法（E203 脚本的 insertion_problems 与 balance_problems）：把这份脚本的整套改动（E203 的加自己的）插完之后，
    每一行插进去的语句要落在语句层，改过的 .rs 文件括号要配平。交回问题条数。"""
    return e203.syntax_report(label, len(e203.edits()) + len(edits()), files, transform(files, "counts"))


def check_anchors(tree, syntax=True):
    """打主工作区那一刻每个 E206 锚点的行号；E203 的锚点由 transform 一起核。锚点全对之后再过不起编译的判法（syntax=False 只给假树的自证用：假树不是合法的 Rust）。"""
    files = e203.read_tree(tree, needed_paths())
    bad = 0
    for edit in edits():
        try:
            position = e203.locate(files[edit.file], edit)
            print(f"name=anchor id={edit.ident} file={edit.file} line={files[edit.file].count(chr(10), 0, position) + 1} hits=1")
        except e203.AnchorError as error:
            print(f"name=anchor id={edit.ident} file={edit.file} problem={error}")
            bad += 1
    try:
        transform(files, "counts")
    except e203.AnchorError as error:
        print(f"name=anchor id=transform problem={error}")
        bad += 1
    if bad == 0 and syntax:
        bad += 1 if syntax_problems(files, "e206_counts") else 0
    return 0 if bad == 0 else 3


FAKE_SETTLE_BODY = (
    "    let (previous_nodes, previous_records): (\n"
    "    let mut rewritten: BTreeSet<AllocationRecordTreeNode> = BTreeSet::new();\n"
    + NODES_AFTER_DECLARED + LIMIT_LOOP + ROUND_BEGIN_ANCHOR
    + "            rewritten.intersection(&nodes_after).copied().collect();\n"
    "        let allocation_plan = crate::allocation_record_tree::plan_the_tree_after_this_publish(\n"
    "        let (previous_roles_replaced, mut release) =\n"
    + REHEARSAL + CHANGED_BY_THE_RELEASE
    + "        if !changed_by_the_release.is_subset(&rewritten_by_the_plan) {\n"
    + RELEASE_MERGE
    + "            continue;\n        }\n"
    "        let changed =\n            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());\n"
    + HOLDING
    + "        if holding_records == nodes_after && changed.is_subset(&rewritten_by_the_plan) {\n"
    "            return Ok(SettledPublish {\n                resolved,\n            });\n        }\n"
    + PLACEMENT_MERGE
    + "    }"
)
FAKE_ROW_SETTLE_BODY = (
    NODES_AFTER_DECLARED + LIMIT_LOOP + ROUND_BEGIN_ANCHOR + REHEARSAL
    + "        if holding_records == nodes_after {\n            return Ok(SettledRowPublishOnAVersionWithoutFile {\n                release,\n            });\n        }\n    }"
)
FAKE_ROW_CALLER_BODY = (
    "    let settled = settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file(\n"
    "        previous_root,\n        plan.instance_table,\n        txg,\n        allocator,\n    )?;"
)


def fake_tree():
    """E203 脚本的假树，把固定点那个函数换成照真实次序排的一份，再添 E206 自己要的调用处与另一份固定点。"""
    files = e203.fake_tree()
    text = files[T]
    start = text.index(HEAD_SETTLE) + len(HEAD_SETTLE)
    stop = text.index(FN_END, start)
    text = text[:start] + FAKE_SETTLE_BODY + text[stop:]
    call = "    let settled = settle_the_allocation_record_tree(\n"
    text = text.replace(call, call + "        pool.snapshot_lifetime_judgement(),\n    )?;\n", 1)
    text += HEAD_ROW_SETTLE + FAKE_ROW_SETTLE_BODY + FN_END + HEAD_ROW_CALLER + FAKE_ROW_CALLER_BODY + FN_END
    files[T] = text
    return files


def write_fake_source(root):
    """一棵小的假仓根（入口脚本的自证拿它跑真的 --build）：E203 脚本的假仓根，再把 E206 改过的文件写上。"""
    e203.write_fake_source(root)
    for relative, text in fake_tree().items():
        (pathlib.Path(root) / relative).parent.mkdir(parents=True, exist_ok=True)
        (pathlib.Path(root) / relative).write_text(text, encoding="utf-8")


def function_body(text, head, end=FN_END):
    start = text.index(head)
    return text[start:text.index(end, start + len(head))]


def calls_on_the_paths(settle_body):
    """照固定点循环的次序走三种出口，交回 {出口: 走到的 e206_fixed_point 调用名}（假树上跑一遍插入之后的循环体，不编）。"""
    loop = settle_body[settle_body.index("rounds_limit() {"):]
    release_if = loop.index("if !changed_by_the_release.is_subset")
    continue_at = loop.index("continue;")
    settled_if = loop.index("if holding_records == nodes_after")
    return_at = loop.index("return Ok(SettledPublish")
    settled_end = loop.index("});\n        }\n", return_at)
    common, release_block = loop[:release_if], loop[release_if:continue_at]
    after_release = loop[continue_at:settled_if]
    settled_block, placement_tail = loop[settled_if:return_at], loop[settled_end:]
    names = lambda text: re.findall(r"e206_fixed_point::([a-z_]+)\(", text)  # noqa: E731
    return {"release_not_covered": names(common + release_block),
            "settled": names(common + after_release + settled_block),
            "placement_not_covered": names(common + after_release + placement_tail)}


def rounds_counted_on(paths, exits):
    """走这一串出口的固定点，计数记下几轮。"""
    return sum(paths[exit_name].count("round_begin") for exit_name in exits)


def selftest():
    failures = []

    def expect(ok, what):
        print(f"name=selftest_case case={what} result={'ok' if ok else 'FAILED'}")
        if not ok:
            failures.append(what)

    files = fake_tree()
    counts = transform(files, "counts")
    module = counts[T][counts[T].index("pub mod e206_fixed_point"):]
    expect("pub mod e203_scan_counts" in counts[T] and "pub mod e206_fixed_point" in counts[T], "counts_has_both_modules")
    expect(transform(files, "plain") == files, "plain_changes_nothing")
    expect(counts[T].count("pub fn between_row_totals") == 1 and counts[T].index("between_row_totals") < counts[T].index("pub mod e206_fixed_point"),
           "the_e203_module_gets_the_accessors")
    paths = calls_on_the_paths(function_body(counts[T], HEAD_SETTLE))
    expect(rounds_counted_on(paths, ["release_not_covered", "placement_not_covered", "settled"]) == 3,
           "three_rounds_are_counted_on_a_fake_fixed_point_with_all_three_exits")
    expect(paths["release_not_covered"] == ["round_begin", "round_resolved", "round_released", "round_after_the_release", "merge_into_the_rewrite_set"],
           "the_release_exit_reads_in_the_order_of_the_loop")
    expect(paths["placement_not_covered"][-2:] == ["round_after_the_placements", "merge_into_the_rewrite_set"],
           "the_placement_exit_goes_through_the_merge_that_walks_the_closure")
    expect(paths["settled"][-2:] == ["round_after_the_placements", "round_settled"], "the_settled_exit_records_the_final_set")
    expect("new_dirty(changed, Some(changed_by_the_release), rewritten_by_the_plan)" in module, "placement_dirty_leaves_out_what_the_release_dirtied")
    walked = function_body(module, "    pub fn closure_is_walked() -> bool {", "\n    }\n")
    expect("Arm::ReleaseChainClosure" in walked and "ClosurePatchedButNotWalked" not in walked, "closure_off_does_not_walk_the_closure")
    expect('"closure_off" => Some(Arm::ClosurePatchedButNotWalked)' in module, "closure_off_is_a_known_arm")
    limit = function_body(module, "    pub fn rounds_limit() -> usize {", "\n    }\n")
    expect("Arm::OneNodeAtATime" in limit and "4096" in limit, "only_single_step_raises_the_limit")
    seed = function_body(module, "    pub fn whole_tree_every() -> Option<u64> {", "\n    }\n")
    expect("Arm::WholeTreeEvery16 => Some(16)" in seed and "Arm::WholeTreeEvery256 => Some(256)" in seed and "Arm::WholeTreeEveryPublish => Some(1)" in seed
           and "Arm::WholeTreePatchedButNever => Some(u64::MAX)" in seed, "the_seeding_arms_have_their_periods")
    row_body = function_body(counts[T], HEAD_ROW_SETTLE)
    expect("merge_into_the_rewrite_set" not in row_body and "seed_the_rewrite_set" not in row_body and "round_settled" in row_body,
           "the_row_publish_fixed_point_is_only_read_not_patched")
    expect("e206_settling_outcome?" in function_body(counts[T], HEAD_JUDGE) and "settle_outcome(&e206_settling_outcome)" in function_body(counts[T], HEAD_ROW_CALLER),
           "both_call_sites_record_the_outcome_before_the_question_mark")
    for name, change in (("an_anchor_lost_stops_the_build", lambda text: text.replace(RELEASE_MERGE, "", 1)),
                         ("an_anchor_of_the_e203_table_lost_stops_the_build", lambda text: text.replace("    let (previous_nodes, previous_records): (\n", "", 1)),
                         ("an_anchor_hit_twice_stops_the_build", lambda text: text.replace(RELEASE_MERGE, RELEASE_MERGE * 2, 1))):
        broken = dict(files)
        broken[T] = change(files[T])
        try:
            transform(broken, "counts")
            expect(False, name)
        except e203.AnchorError:
            expect(True, name)
    with tempfile.TemporaryDirectory() as directory:
        root = pathlib.Path(directory)
        write_fake_source(root / "source")
        arguments = {"--build": "counts", "--source": str(root / "source"), "--into": str(root / "counts"), "--diff-out": str(root / "counts.diff")}
        expect(build(arguments) == 0 and build(arguments) == 0, "the_counts_build_runs_and_runs_again")
        readings = (root / "counts" / READINGS_IN_THE_COPY).read_text(encoding="utf-8")
        expect("pub fn take_settle_lines" in readings and "e206_fixed_point" in readings, "the_counts_copy_has_the_readings")
        expect("e206_fixed_point" in (root / "counts.diff").read_text(encoding="utf-8"), "the_diff_is_written")
        expect(build({"--build": "plain", "--source": str(root / "source"), "--into": str(root / "plain")}) == 0, "the_plain_build_runs")
        plain_core = (root / "plain" / T).read_text(encoding="utf-8")
        plain_readings = (root / "plain" / READINGS_IN_THE_COPY).read_text(encoding="utf-8")
        expect("e206_fixed_point" not in plain_core and "e203_scan" not in plain_core, "the_plain_copy_has_no_counters")
        expect('"plain"' in plain_readings and "singlefs_core::transaction::e206_fixed_point" not in plain_readings, "the_plain_copy_has_the_empty_readings")
        expect(check_anchors(root / "source", syntax=False) == 0, "the_anchor_check_passes_on_the_fake_source")
        expect(check_anchors(root / "source") == 3, "the_anchor_check_runs_the_syntax_check_and_stops_on_a_tree_that_is_not_rust")
    print(f"name=selftest_done failures={len(failures)}")
    return 0 if not failures else 4


def lay_out_copy_beside_siblings(root, real_directory):
    """变异副本要从临时目录起：按本脚本的写法兄弟脚本取自副本旁边，所以把兄弟文件与上一层的 hosts 都链进临时目录里同形的位置，返回副本路径。"""
    beside = root / real_directory.name
    beside.mkdir()
    for entry in real_directory.iterdir():
        if entry.is_file():
            (beside / entry.name).symlink_to(entry)
    (root / "hosts").symlink_to(real_directory.parent / "hosts")
    return beside / "mutated.py"


def mutation_check(table):
    """对变异表逐条跑：把原文换成替换文写到临时文件，跑 --selftest，必须退非 0 才算抓到。原文与替换文逐字取自本脚本。"""
    import subprocess
    source = pathlib.Path(__file__).read_text(encoding="utf-8")
    missed = 0
    for line in pathlib.Path(table).read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        name, original, replacement = line.split("\t")[:3]
        if source.count(original) != 1:
            print(f"name=counters_mutation id={name} result=anchor_lost hits={source.count(original)}")
            missed += 1
            continue
        with tempfile.TemporaryDirectory() as directory:
            mutated = lay_out_copy_beside_siblings(pathlib.Path(directory), pathlib.Path(__file__).resolve().parent)
            mutated.write_text(source.replace(original, replacement), encoding="utf-8")
            outcome = subprocess.run([sys.executable, "-I", str(mutated), "--selftest"], capture_output=True, text=True, check=False)
        caught = outcome.returncode != 0
        missed += 0 if caught else 1
        print(f"name=counters_mutation id={name} result={'caught' if caught else 'missed'} exit={outcome.returncode}")
    print(f"name=counters_mutation_done missed={missed}")
    return 0 if missed == 0 else 4


def main(argv):
    if os.environ.get("E206_RUN_BREAK") == e203.COPY_RESUME_OFF:
        os.environ["E203_RUN_BREAK"] = e203.COPY_RESUME_OFF
    args, index = {}, 0
    while index < len(argv):
        if argv[index] == "--selftest":
            args[argv[index]] = True
            index += 1
        elif argv[index].startswith("--") and index + 1 < len(argv):
            args[argv[index]] = argv[index + 1]
            index += 2
        else:
            print(__doc__, file=sys.stderr)
            return 2
    if "--selftest" in args:
        return selftest()
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
