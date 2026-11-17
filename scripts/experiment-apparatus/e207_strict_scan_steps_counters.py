#!/usr/bin/env python3
"""E207（B8严格判据下的步数）量的那一路：在拷来的实现上插固定点的逐轮读数、节点年龄与首次合并，不改 E203、E206 的脚本。

跑前登记 research/prompts/e207-preregistration.md 第三节「每个读数的取样位置」、第五节「装置写在哪」第 1 条。

用法：
  python3 e207_strict_scan_steps_counters.py --build counts|plain --source <仓根> --into <工作目录> [--diff-out <文件>]
  python3 e207_strict_scan_steps_counters.py --check-anchors <树>      只核锚点、打每个锚点的行号，不写；另打 E206 那份插入表在这棵树上的命中（只报，不进退出码）；锚点全对之后再不起编译地判一遍：插完的每行读数语句要落在语句层（前一个字符是 ; { }、最内层括号是 {，不在表达式、实参表、链式调用中间），改过的 .rs 括号要配平，不过退 3、打 name=insertion_problem / name=balance_problem 行（判法在 E203 脚本里）
  python3 e207_strict_scan_steps_counters.py --selftest                用一棵小的假树证每个分支（不编实现）；各格按每台默认重活线程数并行，末行报「并行 N 路」
  python3 e207_strict_scan_steps_counters.py --mutation-check <变异表>  逐条改坏、跑自证，没红的报 missed
  python3 e207_strict_scan_steps_counters.py --write-fake-source <新目录>  写一棵小的假仓根（入口脚本的自证用）
退出码：0 成；2 用法错或文件不在；3 有锚点命中不是恰好一次（一个字节不写，登记停机条款第 1 条）；4 自证有一格不对。

--build counts：先照 e203_admission_scan_cost_counters.py 的编辑表插 E203（准入之前扫类的步代价）的计数（import 它，不改它），再插第三节第 1、2、4 组读数；实现自己的固定点与
  merge_into_the_rewrite_set_walking_the_release_chain 一个字节不改，读数只插在调用前后。臂只有一条（今天的实现），没有补丁。
  读数模块的大部分取自 e206_fixed_point_cost_counters.py 的模块文字（import 它，不改它），去掉臂与补丁那几段，加第 4 组。
  测试目录里写 e207_readings/mod.rs（读数行怎么打，行首 E207RESULT）。
--build plain：只拷不插，测试目录里写另一份 e207_readings/mod.rs（读数全空），给作废条款第 3 条比盘。
每个锚点在它那个函数体里必须恰好命中一次，否则退 3。副本的开拷、完成标记与续拷照 E203 脚本的 prepare_copy、mark_copy_done（同一条命令重起就是续跑）。
弄坏开关（只给自证用，平常跑时不认）：E207_RUN_BREAK=<项> 由入口自证带过来，这份脚本只认 copy-resume-off（照 E203 脚本的 E203_RUN_BREAK=copy-resume-off 转过去）：
  E207_RUN_BREAK=copy-resume-off  不认完成标记与开拷标记，非空目录一律拒
      入口自证的 killed_after_the_copy_and_run_again_completes（失败行句首 name=selftest_case case=<这个名字> result=FAILED）
另一个弄坏开关只给本脚本的 --selftest 用：E207_COUNTERS_BREAK=<项>，平常跑时不认：
  E207_COUNTERS_BREAK=nul-end-marker  切 E206 走链那一段时照修之前的写法，在文字末尾接一个空字节当结尾标记（标记留在插出来的文字里）
      本脚本自证的 the_inserted_text_has_no_nul_byte、the_counts_copy_has_no_nul_byte（失败行句首 name=selftest_case case=<这个名字> result=FAILED）
"""
import concurrent.futures
import importlib.util
import os
import pathlib
import re
import sys
import tempfile

sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "hosts"))
from heavy_threads import default_heavy_threads  # noqa: E402

E206_SCRIPT = pathlib.Path(os.path.join(os.path.dirname(os.path.realpath(__file__)), "e206_fixed_point_cost_counters.py"))
READINGS_IN_THE_COPY = "crates/singlefs-verification/tests/e207_readings/mod.rs"
BUILDS = ("counts", "plain")
READINGS_MODULE = "crate::transaction::e207_strict_scan_steps::"


def load_e206():
    specification = importlib.util.spec_from_file_location("e206_counters_for_e207", os.path.join(os.path.dirname(os.path.realpath(__file__)), "e206_fixed_point_cost_counters.py"))
    if specification is None or not E206_SCRIPT.is_file():
        print(f"E206 的插计数脚本不在：{E206_SCRIPT}", file=sys.stderr)
        sys.exit(2)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


e206 = load_e206()
e203 = e206.e203
TRANSACTION_SOURCE = e203.T
FN_END = e203.FN_END
HEAD_SETTLE = e203.HEAD_SETTLE
HEAD_JUDGE = e203.HEAD_JUDGE
HEAD_ROW_SETTLE = e206.HEAD_ROW_SETTLE
HEAD_ROW_CALLER = e206.HEAD_ROW_CALLER
CODE2 = e206.CODE2
HEAD_CODE_TWO_PLAN = e206.HEAD_CODE_TWO_PLAN
HEAD_ZERO_ROOT = e206.HEAD_ZERO_ROOT

LIMIT_LOOP = "    for round_index in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {\n"

NODES_AFTER_DECLARED = e206.NODES_AFTER_DECLARED
ROUND_BEGIN_ANCHOR = e206.ROUND_BEGIN_ANCHOR
ROUND_RESOLVED = "        let (previous_roles_replaced, mut release) =\n"
REHEARSAL = e206.REHEARSAL
CHANGED_BY_THE_RELEASE = e206.CHANGED_BY_THE_RELEASE
HOLDING = e206.HOLDING
SETTLED = "            return Ok(SettledPublish {\n"
ROW_SETTLED = "            return Ok(SettledRowPublishOnAVersionWithoutFile {\n"
RELEASE_MERGE_CALL = ("            merge_into_the_rewrite_set_walking_the_release_chain(\n                &mut rewritten,\n"
                      "                changed_by_the_release,\n                previous_tree,\n")
RELEASE_MERGE_DONE = "            nodes_after.extend(nodes_holding_records(&geometry, rehearsal.records()));\n"
PLACEMENT_MERGE_CALL = ("        merge_into_the_rewrite_set_walking_the_release_chain(\n            &mut rewritten,\n"
                        "            changed,\n            previous_tree,\n")
PLACEMENT_MERGE_DONE = "        nodes_after = holding_records;\n"
IMPLEMENTATION_MERGE = "merge_into_the_rewrite_set_walking_the_release_chain("
CALL_OPEN = "    let settled = settle_the_allocation_record_tree(\n"
CALL_CLOSE = "        pool.snapshot_lifetime_judgement(),\n    )?;\n"
ROW_CALL_OPEN = "    let settled = settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file(\n"
ROW_CALL_CLOSE = "        plan.instance_table,\n        txg,\n        allocator,\n    )?;\n"
CODE_TWO_KEYS = "    Ok(CodeTwoTreePlan {\n        shape: CodeTwoTreeShape { nodes },\n"
CODE_TWO_ZERO_ROOT = ("    if !keys_after_this_publish.is_empty() {\n"
                      "        return plan_the_tree_after_this_publish(previous, keys_after_this_publish, capacity);\n    }\n")


class AnchorError(e203.AnchorError):
    pass


def replaced_once(text, old, new, where):
    """模块文字取自 E206 脚本：每一处替换必须恰好命中一次，否则当锚点失效（E206 那份文字变了，停下核）。"""
    hits = text.count(old)
    if hits != 1:
        raise AnchorError(f"{where}：取自 E206 脚本的文字里这一段命中 {hits} 次，不是一次：{old.strip()[:80]}")
    return text.replace(old, new, 1)


def cut_between(text, begin, end, inserted, where):
    """把 begin 起、end 之前那一段换成 inserted；begin、end 各恰好命中一次。"""
    for marker in (begin, end):
        if text.count(marker) != 1:
            raise AnchorError(f"{where}：取自 E206 脚本的文字里 {marker.strip()[:60]} 命中 {text.count(marker)} 次，不是一次")
    start, stop = text.index(begin), text.index(end)
    return text[:start] + inserted + text[stop:]


def cut_to_the_end(text, begin, where):
    """把 begin 起到文字末尾那一段去掉；begin 恰好命中一次。不往文字里接任何结尾标记。"""
    if text.count(begin) != 1:
        raise AnchorError(f"{where}：取自 E206 脚本的文字里 {begin.strip()[:60]} 命中 {text.count(begin)} 次，不是一次")
    return text[:text.index(begin)]


COUNTERS_BREAK_VARIABLE = "E207_COUNTERS_BREAK"
NUL_END_MARKER = "nul-end-marker"
SEED_FAMILY = "    /// 改起点那一族"


def walk_without_the_seed_family():
    """E206 走链那一段去掉改起点那一族（它在那一段末尾）。弄坏开关 nul-end-marker 照修之前的写法接空字节当结尾标记。"""
    if os.environ.get(COUNTERS_BREAK_VARIABLE) == NUL_END_MARKER:
        return cut_between(e206.MODULE_PART_WALK + "\x00", SEED_FAMILY, "\x00", "", "seed")
    return cut_to_the_end(e206.MODULE_PART_WALK, SEED_FAMILY, "seed")


MODULE_HEAD = r'''
/// E207（B8严格判据下的步数）只在副本里套上的读数（跑前登记 research/prompts/e207-preregistration.md 第三节）。今天的实现只读、不改。
pub mod e207_strict_scan_steps {
    use super::{PublishError, TransactionUnit};
    use crate::allocation_record_tree::{
        nodes_holding_records, AllocationRecordTreeGeometry, AllocationRecordTreeNode,
        AllocationRecordTreeVersion,
    };
    use crate::allocator::{AllocationRecord, Placement};
    use std::cell::RefCell;
    use std::collections::{BTreeMap, BTreeSet, HashMap};
'''

# 第 4 组：每次固定点多记的几项；首次合并 = 这次固定点里第一次并进重写集的那一处。
SETTLE_RECORD_FIELDS = r'''        /// 这次固定点第一次并进重写集是在哪一处（释放之后或取落点之后）；没并过是 None。
        pub first_merge_site: Option<RoundExit>,
        /// 第一次并进时这一批弄脏、新进重写集的节点数（种子）。
        pub seed_nodes_first_merge: u64,
        /// 第一次并进时顺着释放链新拉进来的节点数（新进重写集、不在这一批弄脏的里）。
        pub pulled_nodes_first_merge: u64,
        /// 拉进来那几个节点的年龄（单位 settle_call）：最小、下中位数（排好序取第 ⌊(n−1)/2⌋ 个）、最大；一个都没有记 0。
        pub pulled_age_minimum: u64,
        pub pulled_age_median: u64,
        pub pulled_age_maximum: u64,
'''
SETTLE_RECORD_INITIAL = r'''                first_merge_site: None,
                seed_nodes_first_merge: 0,
                pulled_nodes_first_merge: 0,
                pulled_age_minimum: 0,
                pulled_age_median: 0,
                pulled_age_maximum: 0,
'''
THREAD_FIELDS = r'''        /// 节点 → 它上一次出现在「定下那一轮」计划重写集里的 settle_call 序号（只在定下处更新）。
        node_last_settled: BTreeMap<AllocationRecordTreeNode, u64>,
'''

MERGE_HOOKS = r'''
    /// 一次并进重写集之前：这一刻的重写集与这一批弄脏的节点（第三节第 4 组：入口时的重写集之差 = 新并进的）。
    pub struct MergeBegin {
        rewritten_before: BTreeSet<AllocationRecordTreeNode>,
        dirty: BTreeSet<AllocationRecordTreeNode>,
    }

    /// 调 `merge_into_the_rewrite_set_walking_the_release_chain` 之前取（`dirty` 那时还没交出去）。
    #[must_use]
    pub fn merge_begin(
        rewritten: &BTreeSet<AllocationRecordTreeNode>,
        dirty: &BTreeSet<AllocationRecordTreeNode>,
    ) -> MergeBegin {
        MergeBegin { rewritten_before: rewritten.clone(), dirty: dirty.clone() }
    }

    /// 节点年龄：此刻的 settle_call 序号减它上一次出现在定下那一轮计划里的序号（没出现过取 0），排好序交回。
    fn sorted_ages(thread: &Thread, nodes: &[AllocationRecordTreeNode]) -> Vec<u64> {
        let mut ages: Vec<u64> = nodes
            .iter()
            .map(|node| thread.settle_calls - thread.node_last_settled.get(node).copied().unwrap_or(0))
            .collect();
        ages.sort_unstable();
        ages
    }

    /// 并完（实现的函数交回之后）：新并进的 = 此刻的重写集减入口时的；其中这一批弄脏的算种子，其余是顺着释放链拉进来的。
    /// 实现的函数里每个新并进的节点恰好进一次待走的那一叠、出栈一次、按上一版查一次指针（没有上一版不走），
    /// 所以跳数取新并进的个数，查找步数照 E206（发布固定点轮数的代价）的口径按上一版节点表里的线性位置算。
    pub fn merge_end(
        begin: MergeBegin,
        rewritten: &BTreeSet<AllocationRecordTreeNode>,
        previous_tree: Option<&AllocationRecordTreeVersion>,
        exit: RoundExit,
    ) {
        let newly: Vec<AllocationRecordTreeNode> =
            rewritten.difference(&begin.rewritten_before).copied().collect();
        let seeds = newly.iter().filter(|node| begin.dirty.contains(*node)).count();
        let pulled: Vec<AllocationRecordTreeNode> =
            newly.iter().filter(|node| !begin.dirty.contains(*node)).copied().collect();
        let (hops, lookup_steps) = previous_tree.map_or((0, 0), |tree| {
            let steps: u64 = newly
                .iter()
                .map(|node| {
                    count(
                        tree.nodes
                            .iter()
                            .position(|(candidate, _)| candidate == node)
                            .map_or(tree.nodes.len(), |found| found + 1),
                    )
                })
                .sum();
            (count(newly.len()), steps)
        });
        let added = count(pulled.len());
        THREAD.with(|thread_cell| {
            let thread = &mut *thread_cell.borrow_mut();
            let ages = sorted_ages(thread, &pulled);
            if let Some(open) = thread.open.as_mut() {
                open.closure_hops += hops;
                open.closure_nodes_added += added;
                open.closure_lookup_steps += lookup_steps;
                if open.first_merge_site.is_none() {
                    open.first_merge_site = Some(exit);
                    open.seed_nodes_first_merge = count(seeds);
                    open.pulled_nodes_first_merge = added;
                    open.pulled_age_minimum = ages.first().copied().unwrap_or(0);
                    open.pulled_age_median = ages.get(ages.len().saturating_sub(1) / 2).copied().unwrap_or(0);
                    open.pulled_age_maximum = ages.last().copied().unwrap_or(0);
                }
            }
            if let Some(round) = thread.current_round.as_mut() {
                round.exit = Some(exit);
                round.closure_hops = hops;
                round.closure_nodes_added = added;
                round.closure_lookup_steps = lookup_steps;
            }
        });
    }

    /// 定下那一轮（`return Ok(SettledPublish` 之前）：这一轮计划重写的每个节点记下此刻的 settle_call 序号；年龄表只在这里更新。
    pub fn note_settled_nodes(rewritten_by_the_plan: &BTreeSet<AllocationRecordTreeNode>) {
        THREAD.with(|thread_cell| {
            let thread = &mut *thread_cell.borrow_mut();
            let call = thread.settle_calls;
            for node in rewritten_by_the_plan {
                thread.node_last_settled.insert(*node, call);
            }
        });
    }

'''


def module_text():
    """读数模块：头（导入，没有臂）+ E206 的一轮走法与入口两个枚举 + E206 的状态、走链与逐轮读数（去掉改起点与并进补丁）+ 第 4 组。"""
    arms = e206.MODULE_PART_ARMS
    exits = arms[arms.index("    /// 一轮走法（读法第 2 条）。"):]
    state = replaced_once(e206.MODULE_PART_STATE, "        pub rounds_detail: Vec<RoundReading>,\n    }\n",
                          "        pub rounds_detail: Vec<RoundReading>,\n" + SETTLE_RECORD_FIELDS + "    }\n", "SettleRecord")
    state = replaced_once(state, "        finished: Vec<SettleRecord>,\n    }\n",
                          "        finished: Vec<SettleRecord>,\n" + THREAD_FIELDS + "    }\n", "Thread")
    walk = walk_without_the_seed_family()
    walk = replaced_once(walk, "                rounds_detail: Vec::new(),\n            });\n",
                         "                rounds_detail: Vec::new(),\n" + SETTLE_RECORD_INITIAL + "            });\n", "settle_begin")
    rounds = cut_between(e206.MODULE_PART_ROUNDS, "    /// 并进重写集（", "    /// 定下（", MERGE_HOOKS, "merge")
    return MODULE_HEAD + exits + state + walk + rounds


class Edit(e203.Edit):
    """一处 E207 的改动（字段同 E203 脚本的 Edit）；只有一种构建插，timing 那一格不用。"""

    def __init__(self, ident, file, head, end, anchor, mode, inserted):
        super().__init__(ident, file, head, end, anchor, mode, inserted, None)


def edits():
    """第三节「每个读数的取样位置」逐个读数的插入点（锚点按今天的 settle_the_allocation_record_tree 与它两处
    merge_into_the_rewrite_set_walking_the_release_chain 的调用重写）。实现的函数本身不动，读数只插在调用前后。"""
    return [
        Edit("call_site_open", TRANSACTION_SOURCE, HEAD_JUDGE, FN_END, CALL_OPEN, "replace",
             "    let e207_settling_outcome = settle_the_allocation_record_tree(\n"),
        Edit("call_site_close", TRANSACTION_SOURCE, HEAD_JUDGE, FN_END, CALL_CLOSE, "replace",
             f"        pool.snapshot_lifetime_judgement(),\n    );\n    {READINGS_MODULE}settle_outcome(&e207_settling_outcome);\n    let settled = e207_settling_outcome?;\n"),
        Edit("settle_begin", TRANSACTION_SOURCE, HEAD_SETTLE, FN_END, NODES_AFTER_DECLARED, "after",
             f"    {READINGS_MODULE}settle_begin({READINGS_MODULE}Entry::Version, &geometry, &previous_nodes, previous_records);\n"),
        Edit("round_begin", TRANSACTION_SOURCE, HEAD_SETTLE, FN_END, ROUND_BEGIN_ANCHOR, "before", f"        {READINGS_MODULE}round_begin(rewritten.len(), nodes_after.len());\n"),
        Edit("round_resolved", TRANSACTION_SOURCE, HEAD_SETTLE, FN_END, ROUND_RESOLVED, "before",
             f"        {READINGS_MODULE}round_resolved(&resolved.rewritten_roles, rewritten_by_the_plan.len());\n"),
        Edit("round_released", TRANSACTION_SOURCE, HEAD_SETTLE, FN_END, REHEARSAL, "before", f"        {READINGS_MODULE}round_released(release.len());\n"),
        Edit("round_after_the_release", TRANSACTION_SOURCE, HEAD_SETTLE, FN_END, CHANGED_BY_THE_RELEASE, "after",
             f"        {READINGS_MODULE}round_after_the_release(&geometry, &changed_by_the_release, &rewritten_by_the_plan, rehearsal.records(), previous_tree);\n"),
        Edit("merge_begin_after_the_release", TRANSACTION_SOURCE, HEAD_SETTLE, FN_END, RELEASE_MERGE_CALL, "before",
             f"            let e207_merge_after_the_release = {READINGS_MODULE}merge_begin(&rewritten, &changed_by_the_release);\n"),
        Edit("merge_end_after_the_release", TRANSACTION_SOURCE, HEAD_SETTLE, FN_END, RELEASE_MERGE_DONE, "before",
             f"            {READINGS_MODULE}merge_end(e207_merge_after_the_release, &rewritten, previous_tree, {READINGS_MODULE}RoundExit::ReleaseNotCovered);\n"),
        Edit("round_after_the_placements", TRANSACTION_SOURCE, HEAD_SETTLE, FN_END, HOLDING, "after",
             f"        {READINGS_MODULE}round_after_the_placements(&changed, &changed_by_the_release, &rewritten_by_the_plan, &holding_records, &nodes_after, rehearsal.records());\n"),
        Edit("round_settled", TRANSACTION_SOURCE, HEAD_SETTLE, FN_END, SETTLED, "before",
             f"            {READINGS_MODULE}round_settled(&resolved.rewritten_roles, &release, rehearsal.records(), previous_records);\n"
             f"            {READINGS_MODULE}note_settled_nodes(&rewritten_by_the_plan);\n"),
        Edit("merge_begin_after_the_placements", TRANSACTION_SOURCE, HEAD_SETTLE, FN_END, PLACEMENT_MERGE_CALL, "before",
             f"        let e207_merge_after_the_placements = {READINGS_MODULE}merge_begin(&rewritten, &changed);\n"),
        Edit("merge_end_after_the_placements", TRANSACTION_SOURCE, HEAD_SETTLE, FN_END, PLACEMENT_MERGE_DONE, "before",
             f"        {READINGS_MODULE}merge_end(e207_merge_after_the_placements, &rewritten, previous_tree, {READINGS_MODULE}RoundExit::PlacementNotCovered);\n"),
        # 另一份固定点（树表 0 条那一版写行）：只记第 1 组
        Edit("row_call_site_open", TRANSACTION_SOURCE, HEAD_ROW_CALLER, FN_END, ROW_CALL_OPEN, "replace",
             "    let e207_settling_outcome = settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file(\n"),
        Edit("row_call_site_close", TRANSACTION_SOURCE, HEAD_ROW_CALLER, FN_END, ROW_CALL_CLOSE, "replace",
             f"        plan.instance_table,\n        txg,\n        allocator,\n    );\n    {READINGS_MODULE}settle_outcome(&e207_settling_outcome);\n    let settled = e207_settling_outcome?;\n"),
        Edit("row_settle_begin", TRANSACTION_SOURCE, HEAD_ROW_SETTLE, FN_END, NODES_AFTER_DECLARED, "after",
             f"    {READINGS_MODULE}settle_begin({READINGS_MODULE}Entry::RowPublish, &geometry, &previous_nodes, previous_records);\n"),
        Edit("row_round_begin", TRANSACTION_SOURCE, HEAD_ROW_SETTLE, FN_END, ROUND_BEGIN_ANCHOR, "before", f"        {READINGS_MODULE}round_begin(rewritten.len(), nodes_after.len());\n"),
        Edit("row_round_resolved", TRANSACTION_SOURCE, HEAD_ROW_SETTLE, FN_END, REHEARSAL, "before", f"        {READINGS_MODULE}round_resolved(&rewritten_roles, rewritten_by_the_plan.len());\n"),
        Edit("row_round_settled", TRANSACTION_SOURCE, HEAD_ROW_SETTLE, FN_END, ROW_SETTLED, "before",
             f"            {READINGS_MODULE}round_settled(&rewritten_roles, &release, rehearsal.records(), previous_records);\n"),
        # 三棵多层码 2 树每轮进出的 key：与 E203 的 code_two_keys_changed 同一处
        Edit("code_two_keys", CODE2, HEAD_CODE_TWO_PLAN, FN_END, CODE_TWO_KEYS, "before",
             f"    {READINGS_MODULE}note_code_two_keys(previous_keys.symmetric_difference(&keys_after_this_publish));\n"),
        Edit("code_two_keys_zero_root", CODE2, HEAD_ZERO_ROOT, FN_END, CODE_TWO_ZERO_ROOT, "after",
             f"    {READINGS_MODULE}note_code_two_keys(previous.keys_in_order().into_iter());\n"),
    ]


READINGS_HEADER_OF_E206 = ("//! E206（发布固定点轮数的代价）`--build counts` 的读数行：第三节每组读数打成 `E206RESULT` 行，"
                           "模型 `e206_fixed_point_cost.rs` 读它们。\n")
READINGS_HEADER = ("//! E207（B8严格判据下的步数）`--build counts` 的读数行：第三节第 1、2、4 组读数打成 `E207RESULT` 行，"
                   "模型 `e207_strict_scan_steps.rs` 读它们（第 3 组声明行由历史用例自己打）。\n")
PLAIN_HEADER_OF_E206 = "//! E206（发布固定点轮数的代价）`--build plain` 的读数：实现一个字节没插，全空。历史用例只比 `image_sha256`（作废条款第 5 条）。\n"
PLAIN_HEADER = "//! E207（B8严格判据下的步数）`--build plain` 的读数：实现一个字节没插，全空。历史用例只比 `image_sha256`（作废条款第 3 条）。\n"
SETTLE_LINE_FORMAT_END = "release_chain_depth={} release_chain_nodes={}\",\n"
SETTLE_LINE_ARGUMENTS_END = "        record.release_chain_nodes\n    )\n"
GROUP_FOUR_FORMAT = (" first_merge_site={} seed_nodes_first_merge={} pulled_nodes_first_merge={}"
                     " pulled_age_minimum={} pulled_age_median={} pulled_age_maximum={}")
GROUP_FOUR_ARGUMENTS = (
    "        record.release_chain_nodes,\n"
    "        record.first_merge_site.map_or(\"none\", |exit| exit.tag()),\n"
    "        record.seed_nodes_first_merge,\n        record.pulled_nodes_first_merge,\n"
    "        record.pulled_age_minimum,\n        record.pulled_age_median,\n        record.pulled_age_maximum\n    )\n"
)


def readings_text(kind):
    """读数行取自 E206 脚本的两份文字：模块名、行首换成 E207 的，settle 行加第 4 组六项。"""
    if kind != "counts":
        return replaced_once(e206.READINGS_PLAIN, PLAIN_HEADER_OF_E206, PLAIN_HEADER, "plain 读数头")
    text = replaced_once(e206.READINGS_COUNTS, READINGS_HEADER_OF_E206, READINGS_HEADER, "counts 读数头") + e206.READINGS_COUNTS_TAIL
    text = replaced_once(text, SETTLE_LINE_FORMAT_END,
                         SETTLE_LINE_FORMAT_END.replace("release_chain_nodes={}", "release_chain_nodes={}" + GROUP_FOUR_FORMAT), "settle 行的格式")
    text = replaced_once(text, SETTLE_LINE_ARGUMENTS_END, GROUP_FOUR_ARGUMENTS, "settle 行的参数")
    return text.replace("e206_fixed_point", "e207_strict_scan_steps").replace("E206RESULT", "E207RESULT")


def needed_paths():
    return sorted(set(e203.needed_paths()) | {edit.file for edit in edits()})


def transform(files, kind):
    """files: {相对路径: 原文}。plain 原样；counts 先走 E203 的 transform（换过锚点的表），再插 E207，有一个锚点不是恰好一次就抛 AnchorError、不写。"""
    if kind == "plain":
        return dict(files)
    result = e203.transform(files, "counts", e203.edits())
    for edit in edits():
        if edit.file not in result:
            raise AnchorError(f"文件不在：{edit.file}")
        e203.locate(result[edit.file], edit)
    for edit in edits():
        result[edit.file] = e203.apply_one(result[edit.file], edit, "counts")
    anchor = e206.E203_MODULE_ANCHOR
    if result[TRANSACTION_SOURCE].count(anchor) != 1:
        raise AnchorError(f"E203 模块里的取数锚点命中 {result[TRANSACTION_SOURCE].count(anchor)} 次，不是一次")
    result[TRANSACTION_SOURCE] = result[TRANSACTION_SOURCE].replace(anchor, e206.E203_MODULE_ACCESSORS + anchor) + module_text()
    return result


def build(options):
    kind, source, into = options["--build"], options["--source"], options["--into"]
    if kind not in BUILDS:
        print("--build 只认 counts 或 plain", file=sys.stderr)
        return 2
    files = e203.read_tree(source, needed_paths())
    try:
        changed = transform(files, kind)
        readings = readings_text(kind)
    except e203.AnchorError as error:
        print(f"锚点不是恰好命中一次，一个字节没写：{error}", file=sys.stderr)
        return 3
    diff_text = e203.unified(files, changed)
    scripts = pathlib.Path(__file__).read_text(encoding="utf-8") + E206_SCRIPT.read_text(encoding="utf-8")
    try:
        e203.check_source_is_complete(source)
        key = e203.copy_key(source, kind, diff_text + readings + scripts)
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
    if "--diff-out" in options:
        e203.write_atomically(options["--diff-out"], diff_text)
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
    """停机条款第 1 条（锚点全对之后再过不起编译的判法；syntax=False 只给假树的自证用：假树不是合法的 Rust）：打这棵树上每个 E207 锚点的行号、E203 表里换过写法的条目，与 E206 那份插入表的命中（只报）。"""
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
        readings_text("counts")
    except e203.AnchorError as error:
        print(f"name=anchor id=transform problem={error}")
        bad += 1
    if bad == 0 and syntax:
        bad += 1 if syntax_problems(files, "e207_counts") else 0
    for edit in e206.edits():
        hits = "-"
        try:
            e203.locate(files[edit.file], edit)
            hits = "1"
        except (e203.AnchorError, KeyError) as error:
            hits = f"not_one ({error})"
        print(f"name=e206_anchor_on_this_tree id={edit.ident} hits={hits}")
    return 0 if bad == 0 else 3


FAKE_SETTLE_BODY = (
    "    let (previous_nodes, previous_records): (\n"
    "    let previous_tree = previous.map(|previous_version| &previous_version.allocation_record_tree);\n"
    "    let mut rewritten: BTreeSet<AllocationRecordTreeNode> = BTreeSet::new();\n"
    + NODES_AFTER_DECLARED + LIMIT_LOOP + ROUND_BEGIN_ANCHOR
    + "            rewritten.intersection(&nodes_after).copied().collect();\n"
    "        let allocation_plan = crate::allocation_record_tree::plan_the_tree_after_this_publish(\n"
    + ROUND_RESOLVED + REHEARSAL + CHANGED_BY_THE_RELEASE
    + "        if !changed_by_the_release.is_subset(&rewritten_by_the_plan) {\n"
    + RELEASE_MERGE_CALL + "                &geometry,\n            );\n" + RELEASE_MERGE_DONE
    + "            continue;\n        }\n"
    "        let changed =\n            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());\n"
    + HOLDING
    + "        if holding_records == nodes_after && changed.is_subset(&rewritten_by_the_plan) {\n"
    + SETTLED + "                resolved,\n            });\n        }\n"
    + PLACEMENT_MERGE_CALL + "            &geometry,\n        );\n" + PLACEMENT_MERGE_DONE
    + "    }"
)
FAKE_ROW_SETTLE_BODY = (
    NODES_AFTER_DECLARED + LIMIT_LOOP + ROUND_BEGIN_ANCHOR + REHEARSAL
    + "        if holding_records == nodes_after {\n" + ROW_SETTLED + "                release,\n            });\n        }\n    }"
)
FAKE_ROW_CALLER_BODY = ROW_CALL_OPEN + "        previous_root,\n" + ROW_CALL_CLOSE.rstrip("\n")


def fake_tree():
    """E203 脚本的假树，把固定点那个函数换成照今天次序排的一份，再添调用处与另一份固定点。"""
    files = e203.fake_tree()
    text = files[TRANSACTION_SOURCE]
    start = text.index(HEAD_SETTLE) + len(HEAD_SETTLE)
    stop = text.index(FN_END, start)
    text = text[:start] + FAKE_SETTLE_BODY + text[stop:]
    text = text.replace(CALL_OPEN, CALL_OPEN + CALL_CLOSE, 1)
    text += HEAD_ROW_SETTLE + FAKE_ROW_SETTLE_BODY + FN_END + HEAD_ROW_CALLER + FAKE_ROW_CALLER_BODY + FN_END
    files[TRANSACTION_SOURCE] = text
    return files


def write_fake_source(root):
    """一棵小的假仓根（入口脚本的自证拿它跑真的 --build）：E203 脚本的假仓根，再把这里改过的文件写上。"""
    e203.write_fake_source(root)
    for relative, text in fake_tree().items():
        (pathlib.Path(root) / relative).parent.mkdir(parents=True, exist_ok=True)
        (pathlib.Path(root) / relative).write_text(text, encoding="utf-8")


def function_body(text, head, end=FN_END):
    start = text.index(head)
    return text[start:text.index(end, start + len(head))]


def calls_on_the_paths(settle_body):
    """照固定点循环的次序走三种出口，交回 {出口: 走到的 e207_strict_scan_steps 调用名}（假树上跑一遍插入之后的循环体，不编）。"""
    loop = settle_body[settle_body.index(LIMIT_LOOP.strip()):]
    release_if = loop.index("if !changed_by_the_release.is_subset")
    continue_at = loop.index("continue;")
    settled_if = loop.index("if holding_records == nodes_after")
    return_at = loop.index("return Ok(SettledPublish")
    settled_end = loop.index("});\n        }\n", return_at)
    common, release_block = loop[:release_if], loop[release_if:continue_at]
    after_release = loop[continue_at:settled_if]
    settled_block, placement_tail = loop[settled_if:settled_end], loop[settled_end:]
    names = lambda text: re.findall(r"e207_strict_scan_steps::([a-z_]+)\(", text)  # noqa: E731
    return {"release_not_covered": names(common + release_block),
            "settled": names(common + after_release + settled_block),
            "placement_not_covered": names(common + after_release + placement_tail)}


def selftest_inserted_text(expect):
    """格：假树上插出来的文字（每条插入的位置、顺序与内容）；只读假树，不碰盘。"""
    files = fake_tree()
    counts = transform(files, "counts")
    module = counts[TRANSACTION_SOURCE][counts[TRANSACTION_SOURCE].index("pub mod e207_strict_scan_steps"):]
    expect("pub mod e203_scan_counts" in counts[TRANSACTION_SOURCE] and "pub mod e207_strict_scan_steps" in counts[TRANSACTION_SOURCE] and "e206_fixed_point" not in counts[TRANSACTION_SOURCE],
           "counts_has_the_e203_and_e207_modules_only")
    expect(transform(files, "plain") == files, "plain_changes_nothing")
    expect(counts[TRANSACTION_SOURCE].count("pub fn between_row_totals") == 1 and counts[TRANSACTION_SOURCE].index("between_row_totals") < counts[TRANSACTION_SOURCE].index("pub mod e207_strict_scan_steps"),
           "the_e203_module_gets_the_accessors")
    paths = calls_on_the_paths(function_body(counts[TRANSACTION_SOURCE], HEAD_SETTLE))
    expect(sum(paths[exit_name].count("round_begin") for exit_name in paths) == 3, "three_rounds_are_counted_on_a_fake_fixed_point_with_all_three_exits")
    expect(paths["release_not_covered"] == ["round_begin", "round_resolved", "round_released", "round_after_the_release", "merge_begin", "merge_end"],
           "the_release_exit_reads_in_the_order_of_the_loop")
    expect(paths["placement_not_covered"][-3:] == ["round_after_the_placements", "merge_begin", "merge_end"],
           "the_placement_exit_reads_around_the_implementation_merge")
    expect(paths["settled"][-3:] == ["round_after_the_placements", "round_settled", "note_settled_nodes"], "the_settled_exit_records_the_final_set")
    expect(all("note_settled_nodes" not in paths[exit_name] for exit_name in ("release_not_covered", "placement_not_covered")),
           "the_age_table_is_updated_only_on_the_settled_exit")
    expect(function_body(counts[TRANSACTION_SOURCE], HEAD_SETTLE).count(IMPLEMENTATION_MERGE) == FAKE_SETTLE_BODY.count(IMPLEMENTATION_MERGE) == 2
           and "pub fn merge_into_the_rewrite_set(" not in module and "fn seed_the_rewrite_set" not in module,
           "the_implementation_walk_is_called_as_it_is_and_no_patch_is_left")
    for site in ("RELEASE", "PLACEMENT"):
        call = RELEASE_MERGE_CALL if site == "RELEASE" else PLACEMENT_MERGE_CALL
        done = RELEASE_MERGE_DONE if site == "RELEASE" else PLACEMENT_MERGE_DONE
        body = function_body(counts[TRANSACTION_SOURCE], HEAD_SETTLE)
        begin_at, call_at, end_at, done_at = (body.index(f"e207_merge_after_the_{site.lower()}"), body.index(call),
                                               body.index(f"merge_end(e207_merge_after_the_{site.lower()}"), body.index(done))
        expect(begin_at < call_at < end_at < done_at, f"the_merge_after_the_{site.lower()}_is_read_before_and_after_the_call")
    expect("new_dirty(changed, Some(changed_by_the_release), rewritten_by_the_plan)" in module, "placement_dirty_leaves_out_what_the_release_dirtied")
    expect("if open.first_merge_site.is_none()" in module and "ages.get(ages.len().saturating_sub(1) / 2)" in module,
           "the_first_merge_is_kept_and_the_median_is_the_lower_one")
    row_body = function_body(counts[TRANSACTION_SOURCE], HEAD_ROW_SETTLE)
    expect("merge_begin" not in row_body and "round_settled" in row_body and "note_settled_nodes" not in row_body,
           "the_row_publish_fixed_point_is_only_read_for_the_first_group")
    expect("e207_settling_outcome?" in function_body(counts[TRANSACTION_SOURCE], HEAD_JUDGE) and "settle_outcome(&e207_settling_outcome)" in function_body(counts[TRANSACTION_SOURCE], HEAD_ROW_CALLER),
           "both_call_sites_record_the_outcome_before_the_question_mark")
    readings = readings_text("counts")
    expect("E206RESULT" not in readings and "e206_fixed_point" not in readings and "pulled_age_median={}" in readings
           and "record.first_merge_site.map_or" in readings, "the_counts_readings_print_the_fourth_group_under_e207")
    expect(all("\x00" not in text for text in counts.values()) and "\x00" not in module_text() and "\x00" not in readings,
           "the_inserted_text_has_no_nul_byte")


ANCHOR_BREAKS = (("an_anchor_lost_stops_the_build", lambda text: text.replace(RELEASE_MERGE_DONE, "", 1)),
                 ("an_anchor_of_the_e203_table_lost_stops_the_build", lambda text: text.replace("    let (previous_nodes, previous_records): (\n", "", 1)),
                 ("an_anchor_hit_twice_stops_the_build", lambda text: text.replace(PLACEMENT_MERGE_CALL, PLACEMENT_MERGE_CALL * 2, 1)),
                 ("a_loop_head_that_is_not_in_the_tree_stops_the_build",
                  lambda text: text.replace(LIMIT_LOOP, "    for _round in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {\n")))


def selftest_anchor_break(expect, name, change):
    """格：一个锚点改坏的假树，构建要抛 AnchorError；只读假树，不碰盘。"""
    files = fake_tree()
    broken = dict(files)
    broken[TRANSACTION_SOURCE] = change(files[TRANSACTION_SOURCE])
    try:
        transform(broken, "counts")
        expect(False, name)
    except e203.AnchorError:
        expect(True, name)


def selftest_build_copies(expect):
    """格：在自己的临时目录里把假仓根造出来，构建 counts 两遍与 plain，读副本。"""
    with tempfile.TemporaryDirectory() as directory:
        root = pathlib.Path(directory)
        write_fake_source(root / "source")
        arguments = {"--build": "counts", "--source": str(root / "source"), "--into": str(root / "counts"), "--diff-out": str(root / "counts.diff")}
        expect(build(arguments) == 0 and build(arguments) == 0, "the_counts_build_runs_and_runs_again")
        copied = (root / "counts" / READINGS_IN_THE_COPY).read_text(encoding="utf-8")
        expect("pub fn take_settle_lines" in copied and "e207_strict_scan_steps" in copied, "the_counts_copy_has_the_readings")
        expect("e207_strict_scan_steps" in (root / "counts.diff").read_text(encoding="utf-8"), "the_diff_is_written")
        expect(all(b"\x00" not in path.read_bytes() for path in (root / "counts").rglob("*") if path.is_file()),
               "the_counts_copy_has_no_nul_byte")
        expect(build({"--build": "plain", "--source": str(root / "source"), "--into": str(root / "plain")}) == 0, "the_plain_build_runs")
        plain_core = (root / "plain" / TRANSACTION_SOURCE).read_text(encoding="utf-8")
        plain_readings = (root / "plain" / READINGS_IN_THE_COPY).read_text(encoding="utf-8")
        expect("e207_strict_scan_steps" not in plain_core and "e203_scan" not in plain_core, "the_plain_copy_has_no_counters")
        expect('"plain"' in plain_readings and "singlefs_core::transaction::e207_strict_scan_steps" not in plain_readings, "the_plain_copy_has_the_empty_readings")


def selftest_anchor_check(expect):
    """格：在自己的临时目录里核锚点：完整的假树过、丢一个锚点的假树退 3。"""
    with tempfile.TemporaryDirectory() as directory:
        root = pathlib.Path(directory)
        write_fake_source(root / "source")
        expect(check_anchors(root / "source", syntax=False) == 0, "the_anchor_check_passes_on_the_fake_source")
        expect(check_anchors(root / "source") == 3, "the_anchor_check_runs_the_syntax_check_and_stops_on_a_tree_that_is_not_rust")
        write_fake_source(root / "lost")
        lost = root / "lost" / TRANSACTION_SOURCE
        lost.write_text(lost.read_text(encoding="utf-8").replace(RELEASE_MERGE_DONE, "", 1), encoding="utf-8")
        expect(check_anchors(root / "lost", syntax=False) == 3, "the_anchor_check_stops_on_a_lost_anchor")


def selftest():
    """各格互不依赖（各造各的假树、各用各的临时目录），按每台默认重活线程数并行跑；结果按格的固定次序打。"""
    cells = [selftest_inserted_text]
    cells += [(lambda expect, name=name, change=change: selftest_anchor_break(expect, name, change)) for name, change in ANCHOR_BREAKS]
    cells += [selftest_build_copies, selftest_anchor_check]

    def run_cell(cell):
        results = []
        cell(lambda ok, what: results.append((bool(ok), what)))
        return results

    threads = max(1, min(default_heavy_threads(os.environ), len(cells)))
    with concurrent.futures.ThreadPoolExecutor(max_workers=threads) as pool:
        batches = list(pool.map(run_cell, cells))
    failures = []
    for batch in batches:
        for ok, what in batch:
            print(f"name=selftest_case case={what} result={'ok' if ok else 'FAILED'}")
            if not ok:
                failures.append(what)
    print(f"name=selftest_done failures={len(failures)} 并行 {threads} 路（{len(cells)} 格）")
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
    if os.environ.get("E207_RUN_BREAK") == e203.COPY_RESUME_OFF:
        os.environ["E203_RUN_BREAK"] = e203.COPY_RESUME_OFF
    options, index = {}, 0
    while index < len(argv):
        if argv[index] == "--selftest":
            options[argv[index]] = True
            index += 1
        elif argv[index].startswith("--") and index + 1 < len(argv):
            options[argv[index]] = argv[index + 1]
            index += 2
        else:
            print(__doc__, file=sys.stderr)
            return 2
    if "--selftest" in options:
        return selftest()
    os.environ.pop(COUNTERS_BREAK_VARIABLE, None)
    if "--check-anchors" in options:
        return check_anchors(options["--check-anchors"])
    if "--write-fake-source" in options:
        write_fake_source(options["--write-fake-source"])
        return 0
    if "--mutation-check" in options:
        return mutation_check(options["--mutation-check"])
    if all(key in options for key in ("--build", "--source", "--into")):
        return build(options)
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
