import pathlib
import shutil
from e203_admission_scan_cost_counters_applying_edits_and_copying_the_tree import AnchorError, COPIED_DIRECTORIES, COPIED_FILES, COPY_BEGUN, COPY_DONE, CopyError, apply_one, balance_problems, check_builds_are_not_mixed, copy_key, copy_tree, insertion_problems, manifest_paths_not_in, mark_copy_done, module_text, prepare_copy, syntax_gate, transform, unified
from e203_admission_scan_cost_counters_insertion_edits import A, ALLOCATOR_HELPERS, FN_END, GUARD_SCOPE, HARNESS_FEATURES_FROM, HEAD_AFTER_A_CHECK, HEAD_JUDGE, HEAD_SETTLE, HEAD_ZERO_ROOT, METHOD_END, SPAN_NAMES, T, WHOLE_COPY_COUNTED, WHOLE_COPY_NOT_COUNTED, WHOLE_COPY_STRUCTS, edits, whole_copy_fields_named_by_the_helpers, whole_copy_gaps
from e203_admission_scan_cost_counters_row_table_and_module_template import CORE, Edit, P, ROWS, seg


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


SLOT_PAIR_METHOD_TODAY = (
    "impl DeviceFreeMap {\n"
    "    pub fn unblocked_slot_pairs_outside_the_cluster_segments(\n"
    "        &self,\n"
    "        cluster_segments: &BTreeSet<SlotNumber>,\n"
    "    ) -> SlotPairCount {\n"
    "        SlotPairCount(\n"
    "            self.unblocked_slot_pairs.0\n"
    "                - pairs_inside_the_cluster_segments(\n"
    "                    self.unit_area_start,\n"
    "                    &self.unblocked_slot_pairs_per_segment,\n"
    "                    cluster_segments,\n"
    "                )\n"
    "        )\n"
    "    }\n"
    "}\n"
)


def selftest_insertions(expect):
    """格：不起编译的判法（insertion_problems、balance_problems）。每格喂一小段 Rust，要报或不报；
    其中两格喂今天聚簇段求和那个方法体的原样（`SlotPairCount(` 把求和包进实参里），用本表里真的那条改动落上去，锚点落在实参里要红。"""
    ours = "crate::transaction::e203_scan::call(X);\n"

    def problems(body):
        return insertion_problems({"f.rs": "fn f() {\n" + body + "}\n"})[1]

    expect(not problems("    let a = 1;\n    " + ours + "    let b = 2;\n"), "an_inserted_statement_between_two_statements_is_fine")
    expect(not problems('    let s = ")";\n    let c = \'(\';\n    // ]\n    let r = r#"}"#;\n    ' + ours), "brackets_inside_strings_chars_and_comments_are_not_counted")
    # 不配平的括号放进各种「不是代码」的位置，紧跟着一条我们插的语句：把它当代码，栈里就多一个 ( ，插入语句会被误报在括号里
    expect(not problems("    // (\n    " + ours), "an_unbalanced_bracket_in_a_line_comment_right_before_the_insertion_is_not_counted")
    expect(balance_problems({"a.rs": "fn f() {\n}\n"}, {"a.rs": "fn f() {\n    // (\n    g();\n}\n"}) == [], "an_unbalanced_bracket_in_a_line_comment_does_not_break_the_balance")
    expect(not problems("    /* ( */\n    " + ours), "an_unbalanced_bracket_in_a_block_comment_right_before_the_insertion_is_not_counted")
    expect(not problems('    let s = "(";\n    ' + ours), "an_unbalanced_bracket_in_a_string_right_before_the_insertion_is_not_counted")
    expect(balance_problems({"a.rs": "fn f() {\n}\n"}, {"a.rs": "fn f<'a, 'b>(x: &'a(u8), y: &'b u8) {\n    g();\n}\n"}) == [], "a_lifetime_mark_is_not_taken_for_a_char_literal")
    expect(bool(problems("    call(\n        a;\n        " + ours + "    )\n")), "an_inserted_statement_inside_an_argument_list_is_reported")
    expect(bool(problems("    let s = Foo {\n        a: 1,\n        " + ours + "    };\n")), "an_inserted_statement_after_a_comma_is_reported")
    expect(bool(problems("    self.records\n        .iter()\n        let e207_x = g();\n")), "an_inserted_let_in_the_middle_of_a_method_chain_is_reported")
    today = next(edit for edit in edits() if edit.ident == "s14_cluster_segment_sum")
    old = Edit("old_anchor", A, today.head, today.end, "        self.unblocked_slot_pairs.0\n", "before", today.counts, today.timing)
    for kind, case_name in (("counts", "the_cluster_segment_sum_anchor_lands_on_a_statement_boundary_in_todays_method_body_counts"),
                            ("timing", "the_cluster_segment_sum_anchor_lands_on_a_statement_boundary_in_todays_method_body_timing")):
        landed = {A: apply_one(SLOT_PAIR_METHOD_TODAY, today, kind)}
        expect(insertion_problems(landed)[1] == [] and landed[A] != SLOT_PAIR_METHOD_TODAY, case_name)
    landed_old = {A: apply_one(SLOT_PAIR_METHOD_TODAY, old, "counts")}
    expect(len(insertion_problems(landed_old)[1]) == 2 and balance_problems({A: SLOT_PAIR_METHOD_TODAY}, landed_old) == [],
           "the_old_cluster_segment_sum_anchor_inside_the_slot_pair_argument_is_reported_though_the_brackets_balance")
    expect(balance_problems({"a.rs": "fn f() {\n}\n"}, {"a.rs": "fn f() {\n    g();\n}\n"}) == [], "a_balanced_file_after_the_insertion_is_fine")
    expect(len(balance_problems({"a.rs": "fn f() {\n}\n"}, {"a.rs": "fn f() {\n    g(;\n}\n"})) == 1, "an_unclosed_bracket_after_the_insertion_is_reported")
    expect(len(balance_problems({"a.rs": "fn f() {\n}\n"}, {"a.rs": "fn f() {\n    g();)\n}\n"})) == 1, "an_extra_closing_bracket_after_the_insertion_is_reported")
    expect(len(balance_problems({"a.rs": "fn f() {\n"}, {"a.rs": "fn f() {\n    g();\n"})) == 2, "a_file_that_was_not_balanced_before_is_reported_too")
    expect(syntax_gate(fake_tree()) > 0, "the_syntax_gate_of_the_anchor_check_reports_on_a_tree_that_is_not_rust")


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
        ("s26_segments", A, "    pub fn lowest_user_data_slot_outside_the_region(\n", METHOD_END, f"{P}call({P}Row::UserDataSlotLinearScan);",
         ".find_map(|segment_index| {",
         f"{P}scale_to_the_hit({P}Row::UserDataSlotLinearScan, self.segments_with_an_unblocked_slot_pair.len()"
         ' + usize::try_from(CLUSTER_SEGMENT_SLOTS / 2).expect("32"));'),
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
    # s26：产品路径的两个循环（按段走、段内按对找）各记一步，且 `#[cfg(test)]` 下逐槽走的参照实现里一个计数点也没有
    user_slot_step = f"{P}step_to_the_hit({P}Row::UserDataSlotLinearScan);"
    user_slot_loops = (("    pub fn lowest_user_data_slot_outside_the_region(\n", "            .find_map(|segment_index| {\n"),
                       ("    fn lowest_unblocked_slot_pair_in_the_segment(\n", "            .find(|first_index_of_the_pair| {\n"))
    allocator_text = counts[A]
    loops_step = all(allocator_text.count(head) == 1 and (lambda body: body.count(opener + "                " + user_slot_step + "\n") == 1)(
        allocator_text[allocator_text.index(head):allocator_text.index(METHOD_END, allocator_text.index(head) + len(head)) + 1]) for head, opener in user_slot_loops)
    reference_head = "    fn lowest_user_data_slot_outside_the_region_by_stepping_through_every_candidate(\n"
    reference_has_no_counting_site = (reference_head not in allocator_text
                                      or P not in allocator_text[allocator_text.index(reference_head):allocator_text.index(METHOD_END, allocator_text.index(reference_head))])
    expect(loops_step and reference_has_no_counting_site, "the_user_data_slot_scan_counts_one_step_per_segment_and_one_per_pair_on_the_product_path")
    # s37：池级回落按叶取落点的七处步各在自己那个函数体里恰好一处（叶、候选、候选上每块盘的判、逐槽数的槽、槽上每块盘的判、
    # 单元区的叶、叶上每块盘读的没挡槽数），一次调用记在入口、计时守卫只在计时构建里、整个函数体一个
    pooled_step_line = f"{P}step({P}Row::PooledFallbackByTheLeaf);\n"
    pooled_sites = (
        ("    fn commit_generated_fallback_slot_open_on_every_device(\n", "            .find_map(|leaf_index| {\n", 16),
        ("    fn lowest_slot_open_on_every_device_in_the_leaf(\n", "        while candidate.0 + footprint.slots() <= past_the_last_slot_of_the_leaf.0 {\n", 12),
        ("    fn lowest_slot_open_on_every_device_in_the_leaf(\n", "            if self.devices.iter().all(|device| {\n", 16),
        ("    fn slots_open_on_every_device_in_the_leaf(\n", "            .filter(|slot| {\n", 16),
        ("    fn slots_open_on_every_device_in_the_leaf(\n", "                self.devices.iter().all(|device| {\n", 20),
        ("    fn lowest_slot_in_the_leaf_with_the_most_slots_open_on_every_device(\n", "        for leaf_number in first_leaf_index.0..=last_leaf_index.0 {\n", 12),
        ("    fn lowest_slot_in_the_leaf_with_the_most_slots_open_on_every_device(\n", "                .map(|device| {\n", 20),
    )

    def body_of(text, head):
        return text[text.index(head):text.index(METHOD_END, text.index(head) + len(head)) + 1]

    pooled_entry_head = pooled_sites[0][0]
    pooled_call = f"{P}call({P}Row::PooledFallbackByTheLeaf);\n"
    pooled_timer = f"let _e203_timer = {P}timer({P}Row::PooledFallbackByTheLeaf);\n"
    expect(all(counts[A].count(head) == 1 and body_of(counts[A], head).count(opener + " " * indent + pooled_step_line) == 1
               for head, opener, indent in pooled_sites)
           and body_of(counts[A], pooled_entry_head).count(pooled_call) == 1
           and counts[A].count(pooled_step_line) == len(pooled_sites) and counts[A].count(pooled_call) == 1,
           "the_pooled_fallback_counts_one_step_per_leaf_candidate_slot_and_device_check_and_one_call")
    expect(body_of(timing[A], pooled_entry_head).count(pooled_timer) == 1 and pooled_timer not in counts[A]
           and pooled_step_line not in timing[A] and pooled_call not in timing[A],
           "the_pooled_fallback_is_timed_once_over_the_whole_body_in_the_timing_build_only")
    # 整份拷分配器数的集合（s1、s24）：助手文字把「数了的」清单里每个字段都点了名，没点名的字段都写着不数的理由；
    # 字段清单对今天结构体的核对在 `--check-anchors`（今天的树上才有结构体），这里用一份小结构体证核对本身会报多出来的字段与清单里多余的字段
    expect(all(set(WHOLE_COPY_COUNTED[name]) <= whole_copy_fields_named_by_the_helpers(ALLOCATOR_HELPERS, name) for name, _, _ in WHOLE_COPY_STRUCTS)
           and all(not (set(WHOLE_COPY_NOT_COUNTED[name]) & whole_copy_fields_named_by_the_helpers(ALLOCATOR_HELPERS, name))
                   for name, _, _ in WHOLE_COPY_STRUCTS),
           "the_whole_copy_helpers_name_every_collection_that_is_counted_and_none_that_is_not")
    fake_structs = "".join(f"{head}" + "".join(f"    {field}: u64,\n" for field in (*WHOLE_COPY_COUNTED[name], *WHOLE_COPY_NOT_COUNTED[name])) + "}\n"
                           for name, head, _ in WHOLE_COPY_STRUCTS)
    fake_gaps = whole_copy_gaps(fake_structs)
    extra_field = fake_structs.replace("pub struct DeviceFreeMap {\n", "pub struct DeviceFreeMap {\n    a_new_collection: Vec<bool>,\n")
    short_struct = fake_structs.replace("    allocated: u64,\n", "", 1)
    expect(all(unlisted == [] and absent == [] for unlisted, absent in fake_gaps.values())
           and whole_copy_gaps(extra_field)["DeviceFreeMap"] == (["a_new_collection"], [])
           and whole_copy_gaps(short_struct)["DeviceFreeMap"] == ([], ["allocated"]),
           "the_whole_copy_field_check_reports_a_new_field_and_a_field_that_is_gone")
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
    # 每个被量的发布读一次冻结着的发布是不是空：状态行带 `frozen_publish_set`，值取产品里已有的 `allocator.frozen_publish().is_some()`
    # （不空的那一点模型整点作废；变异 c85、c86）
    state_text_start = histories_text.index("fn state_text(")
    state_text_body = histories_text[state_text_start:histories_text.index("\n}\n", state_text_start)]
    expect(state_text_body.count("frozen_publish_set={}") == 1 and state_text_body.count("allocator.frozen_publish().is_some()") == 1,
           "the_state_line_reads_whether_a_publish_is_frozen")
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
    selftest_insertions(expect)
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
