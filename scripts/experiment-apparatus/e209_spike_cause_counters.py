#!/usr/bin/env python3
"""E209（尖峰成因）量的那一路：在拷来的实现上多插节点身份的读数（种子、拉进、定下的节点列表，走释放链的边，上一版每个节点的两份旧落点，
池建好以来带出过的节点），不改 E203、E206、E207 的脚本。

跑前登记 research/prompts/e209-preregistration.md 第五节「交执行员的装置清单」第 1 条。E207 的计数脚本先把它那一套读数插进来（import 它），这里再在插完的文字上打补丁；
读数模块在副本里叫 `e207_strict_scan_steps`（取自 E207 的脚本，是副本里的模块名，不是这个实验的名字），读数行首是 E209RESULT。

用法：
  python3 e209_spike_cause_counters.py --build counts|plain --source <仓根> --into <工作目录> [--diff-out <文件>]
  python3 e209_spike_cause_counters.py --check-anchors <树>      只核锚点、不写（E207 的锚点与这里的补丁锚点都核）
  python3 e209_spike_cause_counters.py --selftest                用一棵小的假树证每个分支（不编实现）；各格按每台默认重活线程数并行
  python3 e209_spike_cause_counters.py --mutation-check <变异表>  逐条改坏、跑自证，没红的报 missed
  python3 e209_spike_cause_counters.py --write-fake-source <新目录>  写一棵小的假仓根（入口脚本的自证用）
退出码：0 成；2 用法错或文件不在；3 有锚点命中不是恰好一次（一个字节不写，登记停机条款第 1 条）；4 自证有一格不对。

--build counts：E207 的 transform（E203 的计数、E206 的走链与逐轮读数、E207 的首次合并与年龄）之后，再打这里的补丁：
  记录里加 seed、pulled、final 三份节点列表、走链的边、released_records_pre、上一版指针图；线程里加池建好以来定下与拉进的节点并集；
  合并结束的钩子多收一个几何参数（走边要用）。实现自己的固定点与 merge_into_the_rewrite_set_walking_the_release_chain 一个字节不改。
  测试目录里写 e209_readings/mod.rs（读数行怎么打，行首 E209RESULT；多 take_node_lines 与 history_carried_line）。
--build plain：只拷不插，测试目录里写另一份 e209_readings/mod.rs（读数全空）。
每个锚点在它那个函数体里必须恰好命中一次，否则退 3。副本的开拷、完成标记与续拷照 E203 脚本（E207 脚本的 build 同一套）。
弄坏开关（只给入口自证用，平常跑时不认）：E209_RUN_BREAK=<项> 由入口自证带过来，这份脚本只认 copy-resume-off（照 E203 脚本的 E203_RUN_BREAK=copy-resume-off 转过去）：
  E209_RUN_BREAK=copy-resume-off  不认完成标记与开拷标记，非空目录一律拒
      入口自证的 killed_after_the_copy_and_run_again_completes（失败行句首 name=selftest_case case=<这个名字> result=FAILED）
"""
import concurrent.futures
import importlib.util
import os
import pathlib
import sys
import tempfile

sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "hosts"))
from heavy_threads import default_heavy_threads  # noqa: E402

E207_SCRIPT = pathlib.Path(os.path.join(os.path.dirname(os.path.realpath(__file__)), "e207_strict_scan_steps_counters.py"))
READINGS_IN_THE_COPY = "crates/singlefs-verification/tests/e209_readings/mod.rs"
BUILDS = ("counts", "plain")
MODULE_MARK = "pub mod e207_strict_scan_steps"
READINGS_MODULE = "crate::transaction::e207_strict_scan_steps::"


def load_e207():
    specification = importlib.util.spec_from_file_location("e207_counters_for_e209", E207_SCRIPT)
    if specification is None or not E207_SCRIPT.is_file():
        print(f"E207 的插计数脚本不在：{E207_SCRIPT}", file=sys.stderr)
        sys.exit(2)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


e207 = load_e207()
e206 = e207.e206
e203 = e207.e203
TRANSACTION_SOURCE = e203.T


class AnchorError(e203.AnchorError):
    pass


def replaced_once(text, old, new, where):
    """每一处替换必须恰好命中一次，否则当锚点失效（取自 E207 脚本的文字变了，停下核）。"""
    hits = text.count(old)
    if hits != 1:
        raise AnchorError(f"{where}：锚点命中 {hits} 次，不是一次：{old.strip()[:80]}")
    return text.replace(old, new, 1)


# ---- 实现一侧的补丁：E207 插进去的两处 merge_end 调用多传几何 ----
CALL_SITE_PATCHES = (
    ("merge_end_after_the_release_gets_the_geometry",
     f"&rewritten, previous_tree, {READINGS_MODULE}RoundExit::ReleaseNotCovered);\n",
     f"&rewritten, previous_tree, &geometry, {READINGS_MODULE}RoundExit::ReleaseNotCovered);\n"),
    ("merge_end_after_the_placements_gets_the_geometry",
     f"&rewritten, previous_tree, {READINGS_MODULE}RoundExit::PlacementNotCovered);\n",
     f"&rewritten, previous_tree, &geometry, {READINGS_MODULE}RoundExit::PlacementNotCovered);\n"),
)

# ---- 读数模块里的补丁（锚点取自 E206、E207 脚本生成的模块文字）----
RECORD_FIELDS_ANCHOR = "        pub pulled_age_maximum: u64,\n"
RECORD_FIELDS = r'''        /// 第一次并进时这一批弄脏、新进重写集的节点（种子）、顺着释放链新拉进来的节点；节点身份照 `node_text`。
        pub seed_list_first_merge: Vec<String>,
        pub pulled_list_first_merge: Vec<String>,
        /// 第一次并进时走释放链的每条边：(拉进者, 被拉进者)。
        pub edges_first_merge: Vec<(String, String)>,
        /// 第一次并进时这里重新走一遍得到的重写集与实现并进之后的重写集是否相同（读数自查）。
        pub edges_walk_matches_the_implementation: bool,
        /// 定下那一轮计划重写的节点。
        pub final_list: Vec<String>,
        /// 进固定点时上一版分配记录里已释放的条数。
        pub released_records_pre: u64,
        /// 上一版每个节点的两份旧落点：(节点, [(盘号, 槽号)])。
        pub pointer_graph: Vec<(String, Vec<(u32, u64)>)>,
        /// 分配记录树的根的层级（几何）。
        pub root_level: u64,
'''
RECORD_INITIAL_ANCHOR = "                pulled_age_maximum: 0,\n"
RECORD_INITIAL = r'''                seed_list_first_merge: Vec::new(),
                pulled_list_first_merge: Vec::new(),
                edges_first_merge: Vec::new(),
                edges_walk_matches_the_implementation: true,
                final_list: Vec::new(),
                released_records_pre: count(previous_records.iter().filter(|record| record.is_released).count()),
                pointer_graph: Vec::new(),
                root_level: 0,
'''
THREAD_FIELDS_ANCHOR = "        node_last_settled: BTreeMap<AllocationRecordTreeNode, u64>,\n"
THREAD_FIELDS = r'''        /// 池建好以来每次发布定下的节点、顺着释放链拉进来的节点（挂载不清零；历史那一段也记），节点身份照 `node_text`。
        carried_ever_final: BTreeSet<String>,
        carried_ever_pulled: BTreeSet<String>,
'''
MERGE_END_SIGNATURE_ANCHOR = "        previous_tree: Option<&AllocationRecordTreeVersion>,\n        exit: RoundExit,\n    ) {\n"
MERGE_END_SIGNATURE = ("        previous_tree: Option<&AllocationRecordTreeVersion>,\n"
                       "        geometry: &AllocationRecordTreeGeometry,\n        exit: RoundExit,\n    ) {\n")
FIRST_MERGE_ANCHOR = "                    open.pulled_age_maximum = ages.last().copied().unwrap_or(0);\n"
FIRST_MERGE = FIRST_MERGE_ANCHOR + r'''                    let walk = edges_of_the_release_chain(&begin.rewritten_before, &begin.dirty, previous_tree, geometry);
                    open.seed_list_first_merge = newly.iter().filter(|node| begin.dirty.contains(*node)).map(node_text).collect();
                    open.pulled_list_first_merge = pulled.iter().map(node_text).collect();
                    open.edges_first_merge = walk.edges;
                    open.edges_walk_matches_the_implementation = walk.reached == *rewritten;
'''
SETTLED_NODES_ANCHOR = ("            for node in rewritten_by_the_plan {\n"
                        "                thread.node_last_settled.insert(*node, call);\n            }\n")
SETTLED_NODES = r'''            for node in rewritten_by_the_plan {
                thread.node_last_settled.insert(*node, call);
                thread.carried_ever_final.insert(node_text(node));
            }
            if let Some(open) = thread.open.as_mut() {
                open.final_list = rewritten_by_the_plan.iter().map(node_text).collect();
            }
            let pulled_now: Vec<String> = thread.open.as_ref().map_or_else(Vec::new, |open| open.pulled_list_first_merge.clone());
            thread.carried_ever_pulled.extend(pulled_now);
'''
POINTER_GRAPH_ANCHOR = "                    open.release_chain_nodes = count(reached.len());\n"
POINTER_GRAPH = POINTER_GRAPH_ANCHOR + ("                    open.pointer_graph = pointer_graph_of(previous_tree);\n"
                                        "                    open.root_level = u64::from(geometry.root_level());\n")
FUNCTIONS_ANCHOR = "    /// 节点年龄：此刻的 settle_call 序号减"
FUNCTIONS = r'''    /// 节点身份的写法：根写 `root`，别的写 `层级/盘号/盘内序号`。
    fn node_text(node: &AllocationRecordTreeNode) -> String {
        match node {
            AllocationRecordTreeNode::Root => "root".to_string(),
            AllocationRecordTreeNode::BelowTheRoot(position) => {
                format!("{}/{}/{}", position.level, position.device.0, position.index_in_device)
            }
        }
    }

    /// 上一版每个节点与它的两份旧落点。
    fn pointer_graph_of(previous_tree: Option<&AllocationRecordTreeVersion>) -> Vec<(String, Vec<(u32, u64)>)> {
        previous_tree.map_or_else(Vec::new, |tree| {
            tree.nodes
                .iter()
                .map(|(node, pointer)| {
                    (node_text(node), pointer.locations.iter().map(|location| (location.device.0, location.slot.0)).collect())
                })
                .collect()
        })
    }

    struct ReleaseChainWalk {
        edges: Vec<(String, String)>,
        reached: BTreeSet<AllocationRecordTreeNode>,
    }

    /// 照实现那一步的走法再走一遍（只多记边：拉进者 → 被拉进者）：从入口时的重写集并上这一批弄脏的节点起，新进来的接着按上一版指针走。
    fn edges_of_the_release_chain(
        rewritten_before: &BTreeSet<AllocationRecordTreeNode>,
        dirty: &BTreeSet<AllocationRecordTreeNode>,
        previous_tree: Option<&AllocationRecordTreeVersion>,
        geometry: &AllocationRecordTreeGeometry,
    ) -> ReleaseChainWalk {
        let mut reached = rewritten_before.clone();
        let mut edges = Vec::new();
        let mut waiting: Vec<AllocationRecordTreeNode> = dirty.iter().copied().filter(|node| reached.insert(*node)).collect();
        let Some(tree) = previous_tree else {
            return ReleaseChainWalk { edges, reached };
        };
        while let Some(node) = waiting.pop() {
            let Some(pointer) = tree.pointer_of(node) else {
                continue;
            };
            for location in pointer.locations {
                let mut climbing = Some(geometry.leaf_of_slot(location.device, location.slot));
                while let Some(ancestor) = climbing {
                    if reached.insert(ancestor) {
                        edges.push((node_text(&node), node_text(&ancestor)));
                        waiting.push(ancestor);
                    }
                    climbing = geometry.parent_of(ancestor);
                }
            }
        }
        ReleaseChainWalk { edges, reached }
    }

    /// 池建好以来已定下的节点与已拉进来的节点（历史那一段的并集，挂载不清零）。
    #[must_use]
    pub fn carried_so_far() -> (Vec<String>, Vec<String>) {
        THREAD.with(|thread_cell| {
            let thread = thread_cell.borrow();
            (thread.carried_ever_final.iter().cloned().collect(), thread.carried_ever_pulled.iter().cloned().collect())
        })
    }

'''
MODULE_PATCHES = (
    ("record_fields", RECORD_FIELDS_ANCHOR, RECORD_FIELDS_ANCHOR + RECORD_FIELDS),
    ("record_initial", RECORD_INITIAL_ANCHOR, RECORD_INITIAL_ANCHOR + RECORD_INITIAL),
    ("thread_fields", THREAD_FIELDS_ANCHOR, THREAD_FIELDS_ANCHOR + THREAD_FIELDS),
    ("merge_end_signature", MERGE_END_SIGNATURE_ANCHOR, MERGE_END_SIGNATURE),
    ("first_merge_lists", FIRST_MERGE_ANCHOR, FIRST_MERGE),
    ("settled_nodes", SETTLED_NODES_ANCHOR, SETTLED_NODES),
    ("pointer_graph_in_the_first_round", POINTER_GRAPH_ANCHOR, POINTER_GRAPH),
    ("node_functions", FUNCTIONS_ANCHOR, FUNCTIONS + FUNCTIONS_ANCHOR),
)


# ---- 读数文件（测试目录里的 e209_readings/mod.rs）----
READINGS_TAKE_ANCHOR = "    for record in e207_strict_scan_steps::take_settles() {\n"
READINGS_TAKE = ("    NODE_ORDER.with(|cell| cell.set(0));\n"
                 "    for record in e207_strict_scan_steps::take_settles() {\n"
                 "        if with_rounds {\n            node_lines_of(item, labels, &record);\n        }\n")
READINGS_NODE_PART = r'''
thread_local! {
    /// `take_settle_lines` 这一趟攒下的节点读数行，由 `take_node_lines` 取走。
    static NODE_LINES: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    /// 这一趟里取走的第几个固定点（重挂载那一趟里按它认写行与暖机）。
    static NODE_ORDER: Cell<u64> = const { Cell::new(0) };
}

/// 取走这一趟攒下的节点读数行（`name=nodes`、`name=pointer_graph`）。
pub fn take_node_lines() -> Vec<String> {
    NODE_LINES.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

fn node_list(nodes: &[String]) -> String {
    if nodes.is_empty() {
        "-".to_string()
    } else {
        nodes.join(",")
    }
}

/// 一个固定点的节点读数：`nodes` 行（种子、拉进、定下的节点列表，走链的边）与 `pointer_graph` 行（上一版每个节点的两份旧落点）。
fn node_lines_of(item: &str, labels: &str, record: &SettleRecord) {
    let order = NODE_ORDER.with(|cell| {
        let order = cell.get();
        cell.set(order + 1);
        order
    });
    let publish_kind = if labels.contains("phase=remount") {
        if order == 0 { "write_row" } else { "warm_up" }
    } else {
        "user"
    };
    let edges: Vec<String> = record.edges_first_merge.iter().map(|(from, to)| format!("{from}>{to}")).collect();
    let graph: Vec<String> = record
        .pointer_graph
        .iter()
        .map(|(node, locations)| {
            let places: Vec<String> = locations.iter().map(|(device, slot)| format!("{device}.{slot}")).collect();
            format!("{node}@{}", places.join("+"))
        })
        .collect();
    NODE_LINES.with(|cell| {
        let mut lines = cell.borrow_mut();
        lines.push(format!(
            "E209RESULT name=nodes item={item} {labels} settle_call={} publish_kind={publish_kind} publish_order={order} first_merge_site={} seed={} pulled={} final={} released_records_pre={} edges={} edges_walk_matches_the_implementation={}",
            record.settle_call,
            record.first_merge_site.map_or("none", |exit| exit.tag()),
            node_list(&record.seed_list_first_merge),
            node_list(&record.pulled_list_first_merge),
            node_list(&record.final_list),
            record.released_records_pre,
            node_list(&edges),
            u8::from(record.edges_walk_matches_the_implementation)
        ));
        lines.push(format!(
            "E209RESULT name=pointer_graph item={item} {labels} settle_call={} publish_kind={publish_kind} publish_order={order} root_level={} graph={}",
            record.settle_call,
            record.root_level,
            node_list(&graph)
        ));
    });
}

/// 历史那一段放完之后：池建好以来已定下、已拉进来的节点（脚本第一步之前的 `C`）。
pub fn history_carried_line(item: &str) -> String {
    let (final_nodes, pulled_nodes) = e207_strict_scan_steps::carried_so_far();
    format!(
        "E209RESULT name=history_carried item={item} ever_final={} ever_pulled={}",
        node_list(&final_nodes),
        node_list(&pulled_nodes)
    )
}
'''
READINGS_PLAIN_PART = r'''
/// plain 版没有节点读数。
pub fn take_node_lines() -> Vec<String> {
    Vec::new()
}
pub fn history_carried_line(_item: &str) -> String {
    String::new()
}
'''


def renamed(text):
    return text.replace("E207RESULT", "E209RESULT").replace("E207（B8严格判据下的步数）", "E209（尖峰成因）").replace("`e207_strict_scan_steps.rs`", "`e209_spike_cause.rs`")


def readings_text(kind):
    """读数行取自 E207 脚本的两份文字：行首换成 E209RESULT，counts 版加节点读数，plain 版加两个空函数。"""
    text = e207.readings_text(kind)
    if kind != "counts":
        return renamed(text) + READINGS_PLAIN_PART
    text = replaced_once(text, READINGS_TAKE_ANCHOR, READINGS_TAKE, "take_settle_lines 取走固定点处")
    return renamed(text) + READINGS_NODE_PART


def needed_paths():
    return e207.needed_paths()


def patch_the_implementation_and_the_module(text, module_patches=MODULE_PATCHES, call_site_patches=CALL_SITE_PATCHES):
    """E207 的 transform 插完之后的 transaction.rs：两处 merge_end 调用多传几何，读数模块打补丁；每个锚点恰好命中一次。"""
    if text.count(MODULE_MARK) != 1:
        raise AnchorError(f"读数模块的开头 {MODULE_MARK} 命中 {text.count(MODULE_MARK)} 次，不是一次")
    start = text.index(MODULE_MARK)
    head, module = text[:start], text[start:]
    for ident, old, new in call_site_patches:
        head = replaced_once(head, old, new, ident)
    for ident, old, new in module_patches:
        module = replaced_once(module, old, new, ident)
    return head + module


def transform(files, kind):
    """files: {相对路径: 原文}。plain 原样；counts 先走 E207 的 transform，再打这里的补丁；有一个锚点不是恰好一次就抛 AnchorError、不写。"""
    result = e207.transform(files, kind)
    if kind == "plain":
        return result
    result[TRANSACTION_SOURCE] = patch_the_implementation_and_the_module(result[TRANSACTION_SOURCE])
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
    scripts = "".join(path.read_text(encoding="utf-8") for path in (pathlib.Path(__file__), E207_SCRIPT, e207.E206_SCRIPT))
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
    edit_count = len(e203.edits()) + len(e207.edits()) + len(CALL_SITE_PATCHES) + len(MODULE_PATCHES) if kind == "counts" else 0
    print(f"name=counters_applied build={kind} edits={edit_count} files={sum(1 for path in changed if changed[path] != files[path])} diff_lines={diff_text.count(chr(10))} copy={state}")
    return 0


def check_anchors(tree, syntax=True):
    """停机条款第 1 条：E207 的锚点（它的 check_anchors）与这里每个补丁锚点各命中几次；锚点全对之后 E207 那一层再过不起编译的判法。"""
    code = e207.check_anchors(tree, syntax)
    files = e203.read_tree(tree, needed_paths())
    try:
        after_e207 = e207.transform(files, "counts")[TRANSACTION_SOURCE]
        patch_the_implementation_and_the_module(after_e207)
        readings_text("counts")
        for ident, _old, _new in CALL_SITE_PATCHES + MODULE_PATCHES:
            print(f"name=anchor_e209 id={ident} hits=1")
    except e203.AnchorError as error:
        print(f"name=anchor_e209 id=transform problem={error}")
        code = 3
    return code


def write_fake_source(root):
    """一棵小的假仓根（入口脚本的自证拿它跑真的 --build）：E207 脚本的假仓根。"""
    e207.write_fake_source(root)


def selftest_patches_apply(expect):
    """格：假树上插出来的文字有每个补丁；plain 不变；两处 merge_end 调用都多传了几何。"""
    files = e207.fake_tree()
    counts = transform(files, "counts")
    text = counts[TRANSACTION_SOURCE]
    module = text[text.index(MODULE_MARK):]
    expect(transform(files, "plain") == files, "plain_changes_nothing")
    expect(text.count("previous_tree, &geometry, crate::transaction::e207_strict_scan_steps::RoundExit::") == 2, "both_merge_ends_get_the_geometry")
    expect("fn edges_of_the_release_chain" in module and "pub fn carried_so_far" in module and "pub final_list: Vec<String>" in module, "the_module_has_the_node_readings")
    expect("carried_ever_final: BTreeSet<String>" in module and "open.pointer_graph = pointer_graph_of(previous_tree)" in module, "the_thread_keeps_the_carried_nodes_and_the_first_round_keeps_the_pointer_graph")
    expect("final_list = rewritten_by_the_plan." in module, "the_settled_exit_keeps_the_final_nodes")
    expect("merge_into_the_rewrite_set_walking_the_release_chain" in function_body_of(text), "the_implementation_walk_is_still_called_as_it_is")
    expect(all("\x00" not in value for value in counts.values()), "the_inserted_text_has_no_nul_byte")


def function_body_of(text):
    start = text.index(e203.HEAD_SETTLE)
    return text[start:text.index(e203.FN_END, start + len(e203.HEAD_SETTLE))]


def selftest_patch_anchor_lost(expect):
    """格：一个补丁锚点命中 0 次或 2 次，构建要抛 AnchorError。"""
    text = transform(e207.fake_tree(), "counts")[TRANSACTION_SOURCE]
    for name, bad in (("a_module_anchor_that_is_gone_stops_the_build", ("lost", "no such text anywhere", "x")),
                      ("a_module_anchor_hit_twice_stops_the_build", ("twice", "pub fn ", "pub fn "))):
        try:
            patch_the_implementation_and_the_module(text, module_patches=MODULE_PATCHES + (bad,))
            expect(False, name)
        except e203.AnchorError:
            expect(True, name)


def selftest_readings(expect):
    """格：counts 的读数行首是 E209RESULT、多节点读数；plain 的两个函数是空的。"""
    counts, plain = readings_text("counts"), readings_text("plain")
    expect("E207RESULT" not in counts and "E207RESULT" not in plain and "E209RESULT name=nodes" in counts, "the_readings_print_under_e209")
    expect("pub fn take_node_lines() -> Vec<String> {\n    NODE_LINES" in counts and "node_lines_of(item, labels, &record)" in counts, "the_counts_readings_take_the_node_lines_with_the_settles")
    expect('if order == 0 { "write_row"' in counts and "publish_order={order}" in counts, "the_remount_publishes_are_told_apart_by_their_order")
    expect("pub fn take_node_lines() -> Vec<String> {\n    Vec::new()" in plain and "history_carried_line(_item: &str) -> String {\n    String::new()" in plain, "the_plain_readings_have_no_node_lines")


def selftest_build_copies(expect):
    """格：在自己的临时目录里把假仓根造出来，构建 counts 两遍与 plain，读副本。"""
    with tempfile.TemporaryDirectory() as directory:
        root = pathlib.Path(directory)
        write_fake_source(root / "source")
        arguments = {"--build": "counts", "--source": str(root / "source"), "--into": str(root / "counts"), "--diff-out": str(root / "counts.diff")}
        expect(build(arguments) == 0 and build(arguments) == 0, "the_counts_build_runs_and_runs_again")
        copied = (root / "counts" / READINGS_IN_THE_COPY).read_text(encoding="utf-8")
        expect("pub fn take_node_lines" in copied and "E209RESULT" in copied, "the_counts_copy_has_the_node_readings")
        expect("edges_of_the_release_chain" in (root / "counts.diff").read_text(encoding="utf-8"), "the_diff_names_the_walk")
        expect(build({"--build": "plain", "--source": str(root / "source"), "--into": str(root / "plain")}) == 0, "the_plain_build_runs")
        plain_core = (root / "plain" / TRANSACTION_SOURCE).read_text(encoding="utf-8")
        expect("e207_strict_scan_steps" not in plain_core and "e203_scan" not in plain_core, "the_plain_copy_has_no_counters")


def selftest_anchor_check(expect):
    """格：核锚点：完整的假树过（不判语法）、丢一个锚点的假树退 3。"""
    with tempfile.TemporaryDirectory() as directory:
        root = pathlib.Path(directory)
        write_fake_source(root / "source")
        expect(check_anchors(root / "source", syntax=False) == 0, "the_anchor_check_passes_on_the_fake_source")
        write_fake_source(root / "lost")
        lost = root / "lost" / TRANSACTION_SOURCE
        lost.write_text(lost.read_text(encoding="utf-8").replace(e207.RELEASE_MERGE_DONE, "", 1), encoding="utf-8")
        expect(check_anchors(root / "lost", syntax=False) == 3, "the_anchor_check_stops_on_a_lost_anchor")


def selftest():
    """各格互不依赖（各造各的假树、各用各的临时目录），按每台默认重活线程数并行跑；结果按格的固定次序打。"""
    cells = [selftest_patches_apply, selftest_patch_anchor_lost, selftest_readings, selftest_build_copies, selftest_anchor_check]

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
            mutated = e207.lay_out_copy_beside_siblings(pathlib.Path(directory), pathlib.Path(__file__).resolve().parent)
            mutated.write_text(source.replace(original, replacement), encoding="utf-8")
            outcome = subprocess.run([sys.executable, "-I", str(mutated), "--selftest"], capture_output=True, text=True, check=False)
        caught = outcome.returncode != 0
        missed += 0 if caught else 1
        print(f"name=counters_mutation id={name} result={'caught' if caught else 'missed'} exit={outcome.returncode}")
    print(f"name=counters_mutation_done missed={missed}")
    return 0 if missed == 0 else 4


def main(argv):
    if os.environ.get("E209_RUN_BREAK") == e203.COPY_RESUME_OFF:
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
