from e203_admission_scan_cost_counters_row_table_and_module_template import CORE, Edit, P, counted_call, guard, row, seg


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
    # 分配器的 newtype 补丁之后这个方法体只剩一个表达式：求和下到自由函数 `pairs_inside_the_cluster_segments`（别的读法也调它，不插在那里），
    # 所以调用与步数记在方法体里、表达式之前（每个聚簇段 1 步，量没变）；计时守卫开在方法体第一句（体里没有别的步）。
    # 锚点取这一整个表达式的起头 `SlotPairCount(`（语句边界上）：取过里面的 `self.unblocked_slot_pairs.0` 那一行，
    # 补丁把它包进 `SlotPairCount(` 的实参里之后，插进去的两句落在实参表中间、编不过（不起编译的判法 insertion_problems 认这一类）。
    add(Edit("s14_cluster_segment_sum", A, "    pub fn unblocked_slot_pairs_outside_the_cluster_segments(\n", METHOD_END,
             "        SlotPairCount(\n", "before",
             counted_call("ClusterSegmentSum")
             + f"        {P}steps({row('ClusterSegmentSum')}, cluster_segments.len());\n",
             guard("ClusterSegmentSum")))
    GUARD_SCOPE["s14_cluster_segment_sum"] = "body"
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
    # s26：产品路径按「还有没挡偶数槽对的 64 槽段」集合升序走（`lowest_user_data_slot_outside_the_region`），走到一段问一次聚簇段集合与 R（1 步，
    # 对应旧口径的「比一个聚簇段」）；落进的那一段里按 2 槽步进逐对看（`lowest_unblocked_slot_pair_in_the_segment`，每对 1 步，对应旧口径的
    # 「看一个候选对」，至多 32 对）。逐槽走的旧实现只在 #[cfg(test)] 下，不插。
    # 上界（调用那一刻读）：集合里的段数（每段至多 1 步）+ 一段里的对数 32（集合里的段至少有一对没挡，第一个没被跳过的段就是答案所在，只有它看对）。
    HEAD_USER_SLOT = "    pub fn lowest_user_data_slot_outside_the_region(\n"
    FIND_SEGMENT = "            .find_map(|segment_index| {\n"
    add(Edit("s26_user_slot_scan", A, HEAD_USER_SLOT, METHOD_END,
             "        self.segments_with_an_unblocked_slot_pair\n            .iter()\n" + FIND_SEGMENT, "before",
             counted_call("UserDataSlotLinearScan")
             + f"        {P}scale_to_the_hit({row('UserDataSlotLinearScan')}, self.segments_with_an_unblocked_slot_pair.len()"
             + " + usize::try_from(CLUSTER_SEGMENT_SLOTS / 2).expect(\"32\"));\n",
             guard("UserDataSlotLinearScan")))
    GUARD_SCOPE["s26_user_slot_scan"] = "body"
    add(Edit("s26_segment_steps", A, HEAD_USER_SLOT, METHOD_END, FIND_SEGMENT, "replace",
             FIND_SEGMENT + f"                {P}step_to_the_hit({row('UserDataSlotLinearScan')});\n", None))
    add(Edit("s26_pair_steps", A, "    fn lowest_unblocked_slot_pair_in_the_segment(\n", METHOD_END,
             "            .find(|first_index_of_the_pair| {\n", "replace",
             "            .find(|first_index_of_the_pair| {\n" + f"                {P}step_to_the_hit({row('UserDataSlotLinearScan')});\n", None))
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
    # s37：池级回落按叶取落点（每块盘都答回落之后，`commit_generated_fallback_slot_open_on_every_device`；每块盘各自先走的 s29 仍照旧记在 s29）。
    # 一步 = 一片叶、一个候选、一个槽、或一次「某块盘在某个候选 / 槽 / 叶上的判」：
    #   重写集里走到的每片叶 1 步；叶里每个候选 1 步、候选上每块盘的判各 1 步；
    #   每块盘都空的单槽数逐槽数一遍：每槽 1 步、槽上每块盘的判各 1 步；
    #   单槽最多的那片叶：单元区罩到的每片叶 1 步（没有提前出口）、叶上每块盘读一次没挡槽数各 1 步（走到的叶里再逐槽数、逐候选找的步记在上面两条）。
    # 每次调用记 1 次调用；整个函数体一个计时守卫（体就是这一整步）。每块盘各取一次最低 / 最高叶序号的那一遍（盘数步）不记。
    HEAD_POOLED_FALLBACK = "    fn commit_generated_fallback_slot_open_on_every_device(\n"
    pooled_step = f"{P}step({row('PooledFallbackByTheLeaf')});\n"
    add(Edit("s37_pooled_fallback_call", A, HEAD_POOLED_FALLBACK, METHOD_END,
             "        rewrite_set_leaves\n            .leaf_indices\n            .iter()\n            .find_map(|leaf_index| {\n", "before",
             counted_call("PooledFallbackByTheLeaf"), guard("PooledFallbackByTheLeaf")))
    GUARD_SCOPE["s37_pooled_fallback_call"] = "body"
    add(Edit("s37_rewrite_set_leaf_steps", A, HEAD_POOLED_FALLBACK, METHOD_END,
             "            .find_map(|leaf_index| {\n", "replace", "            .find_map(|leaf_index| {\n" + f"                {pooled_step}", None))
    add(Edit("s37_candidate_steps", A, "    fn lowest_slot_open_on_every_device_in_the_leaf(\n", METHOD_END,
             "        while candidate.0 + footprint.slots() <= past_the_last_slot_of_the_leaf.0 {\n", "replace",
             "        while candidate.0 + footprint.slots() <= past_the_last_slot_of_the_leaf.0 {\n" + f"            {pooled_step}", None))
    add(Edit("s37_candidate_device_steps", A, "    fn lowest_slot_open_on_every_device_in_the_leaf(\n", METHOD_END,
             "            if self.devices.iter().all(|device| {\n", "replace",
             "            if self.devices.iter().all(|device| {\n" + f"                {pooled_step}", None))
    add(Edit("s37_open_slot_steps", A, "    fn slots_open_on_every_device_in_the_leaf(\n", METHOD_END,
             "            .filter(|slot| {\n                self.devices.iter().all(|device| {\n", "replace",
             "            .filter(|slot| {\n" + f"                {pooled_step}" + "                self.devices.iter().all(|device| {\n"
             + f"                    {pooled_step}", None))
    HEAD_MOST_OPEN_LEAF = "    fn lowest_slot_in_the_leaf_with_the_most_slots_open_on_every_device(\n"
    add(Edit("s37_leaf_loop_steps", A, HEAD_MOST_OPEN_LEAF, METHOD_END,
             "        for leaf_number in first_leaf_index.0..=last_leaf_index.0 {\n", "replace",
             "        for leaf_number in first_leaf_index.0..=last_leaf_index.0 {\n" + f"            {pooled_step}", None))
    add(Edit("s37_leaf_device_steps", A, HEAD_MOST_OPEN_LEAF, METHOD_END,
             "                .map(|device| device.unblocked_slots_in_allocation_record_tree_leaf(leaf_index))\n", "replace",
             "                .map(|device| {\n" + f"                    {pooled_step}"
             + "                    device.unblocked_slots_in_allocation_record_tree_leaf(leaf_index)\n                })\n", None))
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
                  "    for round_index in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {\n",
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
            + self.compaction.clone().take_the_quarantines_noted_by_the_publishes().len()
            + self.root_ring.as_ref().map_or(0, RootRingOccupancy::e203_elements)
            + self
                .allocation_record_tree_of_the_version_without_file
                .as_ref()
                .map_or(0, |version_without_file| {
                    version_without_file.records.len() + version_without_file.version.nodes.len()
                })
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
            + self.segments_with_an_unblocked_slot_pair.len()
            + self.is_under_an_unreleased_record.len()
            + self.slot_pairs_without_an_unreleased_record_per_segment.len()
            + self.unblocked_slots_per_allocation_record_tree_leaf.len()
    }
}

impl RootRingOccupancy {
    fn e203_elements(&self) -> usize {
        self.occupants.len()
            + self.instance_table_judging_the_rereads.rows.len()
            + self
                .placements_named_by_unapplied_journal_records
                .values()
                .map(Vec::len)
                .sum::<usize>()
    }
}
'''
# 整份拷分配器（s1、s24）数的集合：三个结构体的每个字段要么在上面的助手里点了名，要么在这里写明不数的理由。
# `--check-anchors` 拿它对今天 allocator.rs 里三个结构体的字段清单逐个核（字段多了或少了都退 3），自证拿助手文字核点名的那几个。
WHOLE_COPY_STRUCTS = (("DeviceFreeMap", "pub struct DeviceFreeMap {\n", "impl DeviceFreeMap {\n"),
                      ("PoolAllocator", "pub struct PoolAllocator {\n", "impl PoolAllocator {\n"),
                      ("RootRingOccupancy", "pub struct RootRingOccupancy {\n", "impl RootRingOccupancy {\n"))
WHOLE_COPY_COUNTED = {
    "DeviceFreeMap": ("allocated", "isolated", "held_until_floor_takes_effect", "used_per_segment", "unreleased_per_segment",
                      "isolated_per_segment", "held_per_segment", "unblocked_slot_pairs_per_segment", "segments_reclaimed_not_yet_discarded",
                      "segments_with_an_unblocked_slot_pair", "is_under_an_unreleased_record",
                      "slot_pairs_without_an_unreleased_record_per_segment", "unblocked_slots_per_allocation_record_tree_leaf"),
    "PoolAllocator": ("devices", "records", "cluster_segments", "reclaimed", "compaction", "root_ring",
                      "allocation_record_tree_of_the_version_without_file"),
    "RootRingOccupancy": ("occupants", "placements_named_by_unapplied_journal_records", "instance_table_judging_the_rereads"),
}
WHOLE_COPY_SCALAR = "单个数或单个小值，不是集合"
WHOLE_COPY_NOT_COUNTED = {
    "DeviceFreeMap": {name: WHOLE_COPY_SCALAR for name in (
        "device", "device_size_in_bytes", "unit_area_start", "unit_area_slots", "full_segments_without_an_unreleased_record", "free_runs",
        "allocated_slots", "free_slots", "isolated_slots", "deferred_slots", "unblocked_slots", "unblocked_slot_pairs",
        "slot_pairs_without_an_unreleased_record")},
    "PoolAllocator": {
        **{name: WHOLE_COPY_SCALAR for name in (
            "open_segment", "bump_cursor", "format_time_tree_table", "reuse_window",
            "placements_reclaimed_on_release_by_the_forced_zero_reuse_window", "commit_generated_placements_that_fell_back",
            "instance_rows_after_this_mounts_row_publish", "space_admission", "deletion_switches",
            "reclaim_floor_when_the_latest_root_was_recorded")},
        "frozen_publish": "只在一次发布落盘失败、冻结着等重发时是 Some，里面的字段在 transaction.rs 里私有、不数长度；改成每个被量的发布读一次它是不是空"
                          "（状态行的 `frozen_publish_set`，用产品里已有的 `allocator.frozen_publish().is_some()`），不空的那一点整点作废",
        "other_unit_reads_with_one_copy_failing": "Arc：各份拷贝共用同一份，拷贝不拷内容",
    },
    "RootRingOccupancy": {name: WHOLE_COPY_SCALAR for name in ("slots_per_region", "effective_floor", "latest_recorded_txg")},
}


def fields_of_the_struct(text, head):
    import re
    start = text.index(head) + len(head)
    return re.findall(r"^    (?:pub )?([a-z][a-z0-9_]*):", text[start:text.index("\n}\n", start)], re.M)


def whole_copy_gaps(allocator_text):
    """今天 allocator.rs 里三个结构体的字段，对「数了的」「写明不数的」两份清单：交回 {结构体: (没归到任何一份的字段, 清单里有而结构体里没有的字段)}。"""
    gaps = {}
    for struct_name, struct_head, _ in WHOLE_COPY_STRUCTS:
        fields = fields_of_the_struct(allocator_text, struct_head)
        listed = set(WHOLE_COPY_COUNTED[struct_name]) | set(WHOLE_COPY_NOT_COUNTED[struct_name])
        gaps[struct_name] = ([name for name in fields if name not in listed], sorted(listed - set(fields)))
    return gaps


def whole_copy_fields_named_by_the_helpers(helpers_text, struct_name):
    """助手文字里 `impl <结构体> {` 那一块点了名的 `self.<字段>`。"""
    import re
    impl_head = {name: impl for name, _, impl in WHOLE_COPY_STRUCTS}[struct_name]
    start = helpers_text.index(impl_head) + len(impl_head)
    stop = helpers_text.find("\nimpl ", start)
    return set(re.findall(r"self\s*\.\s*([a-z][a-z0-9_]*)", helpers_text[start:stop if stop >= 0 else len(helpers_text)]))


HARNESS_FEATURES_FROM = 'features = ["lookup-work-counters", "test-only-publish-thresholds"]'
HARNESS_FEATURES_TO = 'features = ["test-only-publish-thresholds"]'
