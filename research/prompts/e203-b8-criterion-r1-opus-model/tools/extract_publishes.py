#!/usr/bin/env python3
"""从 E206 单点产物逐次发布抽：B8、B1、B4 三臂的准入之前步数与各种分母的输入。纯文本算术，不起项目代码。
用法：extract_publishes.py <输出 tsv> <产物>...
B8 式子照 scripts/experiment-apparatus/e206_fixed_point_cost.rs 的 by_delta_rows；
B1 取产物 row 行的 steps，只算 E203 ROW_TABLE 里的扫类行；B4 照 e203_admission_scan_cost.rs 的 narrowed_steps_by_segment。"""
import collections
import os
import re
import sys

INODE_LEAF_RECORDS = 233
SEGMENT_SLOTS = 64
SEGMENTS = ("caller", "before_landing", "between")
SCAN_ROWS = set("""whole_allocator_copy_before_the_judgement inode_record_linear_find files_other_than_the_first_plan
file_other_than_the_first_linear_find inodes_with_an_extent_entry_set extent_upper_segment_plan extent_roles_rewritten_scan
root_header_linear_find checkpoint_cost_accounting_tree_plan cluster_segment_sum allocation_record_tree_node_set
allocation_record_tree_plan inode_number_watermark_steps deadlist_changes_indexes multi_level_tree_planning
mapped_units_of_container_trees moved_entries_path_tail_filter release_judgement_version_indexes
whole_allocator_copy_in_a_settling_round allocator_release_linear_find user_data_slot_linear_scan record_intersection_filter
empty_segment_linear_scan fallback_slot_linear_scan records_of_each_leaf accounting_entries_of_this_publish instance_table_chain
mapped_units_whole_table""".split())


def lg(value):
    return value.bit_length()


def levels_64(value):
    level = 1
    while 64 ** level < value:
        level += 1
    return level


def fields(line):
    return dict(token.split("=", 1) for token in line.split()[1:] if "=" in token)


def by_delta_rows(publish, state, segment, segments_per_disk):
    """一段里 B8 每行的步数。"""
    between = segment == "between"
    rounds = publish["rounds"] if between else 0
    deltas = publish["deltas"]
    of_state = lambda key: int(state[key])

    def per_round_maxima(key):
        out = []
        for round_number in range(1, rounds + 1):
            values = [d[3] for d in deltas if d[0] == "between" and d[1] == round_number and d[2] == key]
            out.append(max(values))
        return out

    whole_publish = lambda key: max([d[3] for d in deltas if d[2] == key], default=0)
    in_segment = lambda key: [d[3] for d in deltas if d[0] == segment and d[2] == key]
    cell = lambda row: publish["rows"].get((segment, row), (0, 0, 0))
    calls = lambda row: cell(row)[0]
    touched = whole_publish("touched_files")
    lower = whole_publish("lower_nodes_of_touched_files")
    accounting = whole_publish("accounting_rows_written_by_the_built_on_publish")
    other = of_state("other_files")
    records = of_state("allocation_records")
    mapped = of_state("mapped_units")
    changed_leaves = per_round_maxima("changed_leaves")
    released = per_round_maxima("round_placements_released")
    rewritten = per_round_maxima("round_roles_rewritten")
    keys = in_segment("code_two_keys_changed")
    heights = [of_state("accounting_tree_levels"), of_state("deadlist_tree_levels"), of_state("central_mapping_levels")]
    trees = sum(key * heights[index % 3] for index, key in enumerate(keys))
    leaves_total = sum(changed_leaves)
    tree_levels = of_state("allocation_tree_levels")
    mapped_changed = 2 * sum(rewritten)
    return {
        "whole_allocator_copy_before_the_judgement": 0,
        "inode_record_linear_find": calls("inode_record_linear_find") * (lg(of_state("inode_leaf_containers")) + lg(INODE_LEAF_RECORDS)),
        "files_other_than_the_first_plan": calls("files_other_than_the_first_plan") * touched * (lg(other) + lower),
        "file_other_than_the_first_linear_find": calls("file_other_than_the_first_linear_find") * lg(other),
        "inodes_with_an_extent_entry_set": calls("inodes_with_an_extent_entry_set") * touched * lg(other),
        "extent_upper_segment_plan": calls("extent_upper_segment_plan") * touched * of_state("extent_upper_levels") * lg(of_state("extent_upper_nodes")),
        "extent_roles_rewritten_scan": calls("extent_roles_rewritten_scan") * touched * lower,
        "root_header_linear_find": calls("root_header_linear_find"),
        "checkpoint_cost_accounting_tree_plan": calls("checkpoint_cost_accounting_tree_plan") * accounting * of_state("accounting_tree_levels"),
        "cluster_segment_sum": calls("cluster_segment_sum"),
        "allocation_record_tree_node_set": leaves_total * tree_levels,
        "allocation_record_tree_plan": leaves_total * tree_levels,
        "inode_number_watermark_steps": calls("inode_number_watermark_steps"),
        "deadlist_changes_indexes": cell("deadlist_changes_indexes")[1] + whole_publish("deadlist_entries_changed") * lg(of_state("deadlist_entries")),
        "multi_level_tree_planning": trees,
        "mapped_units_of_container_trees": sum(in_segment("container_units_rewritten")),
        "moved_entries_path_tail_filter": rounds * whole_publish("moved_entries") * of_state("central_mapping_levels"),
        "release_judgement_version_indexes": cell("release_judgement_version_indexes")[1],
        "whole_allocator_copy_in_a_settling_round": sum(a + b for a, b in zip(released, rewritten)),
        "allocator_release_linear_find": calls("allocator_release_linear_find") * lg(records),
        "user_data_slot_linear_scan": calls("user_data_slot_linear_scan") * (levels_64(segments_per_disk) + 32),
        "record_intersection_filter": calls("record_intersection_filter") * lg(records) + sum(in_segment("intersecting_records")),
        "empty_segment_linear_scan": calls("empty_segment_linear_scan") * levels_64(segments_per_disk),
        "fallback_slot_linear_scan": calls("fallback_slot_linear_scan") * (levels_64(segments_per_disk) + 64),
        "records_of_each_leaf": sum(in_segment("dirty_records")),
        "accounting_entries_of_this_publish": calls("accounting_entries_of_this_publish") * accounting,
        "instance_table_chain": calls("instance_table_chain"),
        "mapped_units_whole_table": mapped_changed + mapped_changed * lg(mapped),
    }, released, rewritten


def one_product(path, out):
    point = {}
    publishes = collections.defaultdict(lambda: {"rows": {}, "deltas": [], "rounds": None})
    states = {}
    settles = []
    done = False
    item = None
    for line in open(path, encoding="utf-8"):
        if not line.startswith("E206RESULT "):
            continue
        record = fields(line)
        name = record.get("name")
        if name == "point":
            point = record
        elif name == "item_done":
            done = True
        elif record.get("phase") != "window":
            continue
        elif name == "settle":
            settles.append(record)
        elif name == "state":
            states[(record["state"], record["op"], record["op_index"])] = record
        elif name == "row":
            publishes[record["publish"]]["rows"][(record["segment"], record["row"])] = (int(record["calls"]), int(record["query_levels"]), int(record["steps"]))
        elif name == "delta":
            publishes[record["publish"]]["deltas"].append((record["segment"], int(record["fixed_point_round"]), record["key"], int(record["value"])))
        elif name == "publish":
            publishes[record["publish"]]["rounds"] = int(record["fixed_point_rounds"])
    item = point["item"]
    segments_per_disk = -(-int(point["unit_area_slots"]) // SEGMENT_SLOTS)
    for settle in settles:
        where = (settle["state"], settle["op"], settle["op_index"])
        key = f"{where[0]}/{where[1]}/{where[2]}/{settle['judgement']}"
        publish = publishes[key]
        if publish["rounds"] is None or settle["outcome"] != "settled":
            out.write("\t".join(map(str, ["skipped", item, key, settle["outcome"]])) + "\n")
            continue
        state = states[where]
        b8 = {}
        for segment in SEGMENTS:
            rows, released, rewritten = by_delta_rows(publish, state, segment, segments_per_disk)
            for row, steps in rows.items():
                b8[(segment, row)] = steps
        today = {(s, r): v[2] for (s, r), v in publish["rows"].items() if r in SCAN_ROWS and s in SEGMENTS}
        called = {r for (s, r), v in publish["rows"].items() if r in SCAN_ROWS and s != "between" and s in SEGMENTS and v[0] > 0}
        b4 = sum(v for (s, r), v in b8.items() if s != "between")
        for row in {r for s, r in list(today) + list(b8) if s == "between"}:
            b4 += b8.get(("between", row), 0) if row in called else today.get(("between", row), 0)
        before_second = sum(v for (s, _), v in b8.items() if s != "between")
        b1_before_second = sum(v for (s, _), v in today.items() if s != "between")
        out.write("\t".join(map(str, [
            "publish", item, point["point"], point["arm"], key, settle["state"], settle["op"], settle["op_index"], settle["judgement"],
            publish["rounds"], settle["final_roles"], settle["round_work"], sum(released), released[-1] if released else 0, released[0] if released else 0,
            state["mapped_units"], state["units"], state["allocation_records"], state["other_files"], point["unit_area_slots"], point["unlinks"], point["data_units"], point["files"],
            sum(b8.values()), before_second, sum(today.values()), b1_before_second, b4, int(done),
        ])) + "\n")


def main():
    with open(sys.argv[1], "x", encoding="utf-8") as out:
        out.write("\t".join("kind item point arm publish state op op_index judgement rounds final_roles round_work released_sum released_final released_first mapped units records other_files unit_area unlinks data_units files b8 b8_before_second b1 b1_before_second b4 done".split()) + "\n")
        for path in sys.argv[2:]:
            one_product(path, out)


if __name__ == "__main__":
    main()
