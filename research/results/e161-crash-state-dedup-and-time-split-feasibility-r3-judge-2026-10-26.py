#!/usr/bin/env python3
# E161 第 3 次重跑登记（research/prompts/e161-r3-prereg.md）第六节行 1–3 的判法，只读产物，不起编译、不跑装置。
# 照第 2 次重跑那一份判法（git show 04ceaa904^:research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-judge-2026-10-05.py，
# sha256 2bb9e5fc…）改：对名表 second_segment_16_pc3 ↔ second_segment_18_pc3；行 1 的锚点换成第 3 次登记 A2、A3；Q2c 的数换成第 3 次登记 Q2c。
# 用法：
#   e161-r3-judge.py compare <比对对象（r2 产物）> <这一次的产物>   行 3：Q3a、Q3b、Q3c，按登记 r2 第六节行 3 的字段分类表比方向
#   e161-r3-judge.py pcr3 <比对对象（r2 产物）> <草稿目录>          PCr3：① 自己对自己比；② 改两处的副本对原件比
#   e161-r3-judge.py row-one <这一次的产物>                          行 1：Q1a、Q1b、Q1c（四个数逐段比，F1 那一种单列）
#   e161-r3-judge.py row-two <这一次的产物>                          行 2：Q2a 域大小、Q2c 九样有没有对象
#   e161-r3-judge.py completeness <产物>                             V9 / V10：行数等于收尾行的 emitted，按 name 数行
import os
import sys

SKIPPED_LINE_NAMES = {"config", "done", "preflight", "anchor_segment", "stop_clause_tally"}
KEY_FIELD_NAMES = ["scope", "cell", "stream", "arm", "busy_zone", "side"]
FIELDS_PINNED_AT_ZERO = {
    "inconsistent", "recorded_consult_report_differs", "recorded_ignore_report_differs",
    "recorded_verdicts_differ", "walk_unit_reads_outside_normal_unit_reads", "content_collisions",
    "persisted_mismatches", "second_run_recovery_outcome_changed", "second_run_verdict_changed",
    "pc2a_recovery_keys_changed", "pc4a_checker_keys_changed", "pc2d_recovery_reports_changed",
    "pc4d_verdicts_changed", "inconsistent_without_removal",
}
FIELDS_REQUIRED_POSITIVE = {"constant_key_inconsistent", "hits", "discriminating_positions"}
# 只在字段名上随段号改名的（登记第六节行 3「对名表」）：比对对象那一边的名字 → 这一次的名字。
FIELD_RENAMES = {"second_segment_16_pc3": "second_segment_18_pc3"}
# 第 3 次登记第七节 A2、A3（anchors_e161_r3.py 的原样输出）。
ANCHOR_FIRST_STREAM_SEGMENT_STATES = [8, 8, 3, 1, 8, 3, 1, 8, 16777215, 3, 1, 8]
ANCHOR_SECOND_STREAM_SAMPLED_SEGMENT_STATES = [
    16777215, 3, 1, 8, 8, 8, 262143, 3, 1, 8, 65535, 3, 1, 8, 65535, 3, 1,
]
SECOND_STREAM_SAMPLED_SEGMENTS = range(12, 29)
# 第 3 次登记 Q2a（A4）。
ANCHOR_DOMAIN_SIZES = {
    "first": {"d1_small": 53, "d1_stride": 7260, "d1_quick": 54, "d1_heads": 4096},
    "second": {"d1_small": 393269, "d1_stride": 7260, "d1_quick": 60, "d1_heads": 4096},
}
# 第 3 次登记 Q2c ①：五格 g1 行的 states。
CELL_STATES_EXPECTED = [
    ("first_small", 53), ("first_quick", 54), ("first_segment_head", 4096),
    ("second_quick", 60), ("second_segment_head", 4096),
]
PC3_SEGMENT_EXPECTED = 18


def result_rows_of(path):
    rows = []
    for line_number, line in enumerate(open(path, encoding="utf-8"), 1):
        if not line.startswith("E7RESULT "):
            continue
        fields = {}
        for token in line.rstrip("\n")[len("E7RESULT "):].split(" "):
            if "=" in token:
                field_name, value = token.split("=", 1)
                fields.setdefault(field_name, value)
        rows.append((line_number, fields))
    return rows


def keyed_rows_of(path):
    rows, duplicated_keys = {}, []
    for line_number, fields in result_rows_of(path):
        line_name = fields.get("name")
        if line_name in SKIPPED_LINE_NAMES:
            continue
        key = (line_name,) + tuple(f"{field_name}={fields[field_name]}" for field_name in KEY_FIELD_NAMES if field_name in fields)
        if key in rows:
            duplicated_keys.append((path, line_number, key))
        rows[key] = (line_number, fields)
    return rows, duplicated_keys


def integer_or_none(value):
    try:
        return int(value)
    except ValueError:
        return None


def direction_class(field_name, value, fields):
    if value in ("true", "false"):
        return ("布尔", value)
    number = integer_or_none(value)
    if field_name in FIELDS_PINNED_AT_ZERO and number is not None:
        return ("钉零", "=0" if number == 0 else ">=1")
    if field_name in FIELDS_REQUIRED_POSITIVE and number is not None:
        return ("要正", ">=1" if number >= 1 else "=0")
    if field_name == "distinct_outcomes" and number is not None:
        return ("结局种数", ">=2" if number >= 2 else "<=1")
    if field_name == "states_without_unit_reads" and number is not None:
        states = integer_or_none(fields.get("states", ""))
        if states is not None:
            return ("读不到", "==states" if number == states else "<states")
    if fields.get("name") == "verdict" and len(field_name) == 2 and field_name[0] == "g" and field_name[1].isdigit():
        return ("G行判定", value)
    return None


def compare(earlier_path, later_path):
    earlier_rows, earlier_duplicates = keyed_rows_of(earlier_path)
    later_rows, later_duplicates = keyed_rows_of(later_path)
    for duplicate in earlier_duplicates + later_duplicates:
        print("重复的键", duplicate)
    flips, keys_on_one_side, fields_on_one_side, compared_by_class = [], [], [], {}
    for key in sorted(set(earlier_rows) | set(later_rows), key=lambda key: (key[0], key[1:])):
        if key not in earlier_rows or key not in later_rows:
            keys_on_one_side.append((key, "只在比对对象" if key in earlier_rows else "只在这一次"))
            continue
        earlier_line, earlier_fields = earlier_rows[key]
        later_line, later_fields = later_rows[key]
        earlier_by_name = {FIELD_RENAMES.get(name, name): (name, value) for name, value in earlier_fields.items()}
        later_by_name = {name: (name, value) for name, value in later_fields.items()}
        for field_name in sorted(set(earlier_by_name) | set(later_by_name)):
            if field_name not in earlier_by_name or field_name not in later_by_name:
                fields_on_one_side.append((key, field_name, "只在比对对象" if field_name in earlier_by_name else "只在这一次"))
                continue
            earlier_class = direction_class(earlier_by_name[field_name][0], earlier_by_name[field_name][1], earlier_fields)
            later_class = direction_class(later_by_name[field_name][0], later_by_name[field_name][1], later_fields)
            if earlier_class is None and later_class is None:
                continue
            if earlier_class is None or later_class is None:
                flips.append((key, field_name, earlier_line, earlier_by_name[field_name][1], later_line, later_by_name[field_name][1], "类型对不上"))
                continue
            compared_by_class[earlier_class[0]] = compared_by_class.get(earlier_class[0], 0) + 1
            if earlier_class != later_class:
                flips.append((key, field_name, earlier_line, earlier_by_name[field_name][1], later_line, later_by_name[field_name][1], earlier_class[0]))
    print("键数 比对对象/这一次:", len(earlier_rows), len(later_rows))
    print("比了的字段数（按类）:", compared_by_class, "合计", sum(compared_by_class.values()))
    verdict_flips = [flip for flip in flips if flip[0][0] == "verdict"]
    print("翻面格数:", len(flips), "其中判决行（Q3a）:", len(verdict_flips), "其余（Q3b）:", len(flips) - len(verdict_flips))
    for key, field_name, earlier_line, earlier_value, later_line, later_value, class_name in flips:
        print(f"  翻面 [{class_name}] 键={' '.join(key)} 字段={field_name} 比对对象(行{earlier_line})={earlier_value} 这一次(行{later_line})={later_value}")
    print("只在一边的键数:", len(keys_on_one_side))
    for key, side in keys_on_one_side:
        print("  不可比的键", " ".join(key), side)
    print("只在一边的字段数:", len(fields_on_one_side))
    for key, field_name, side in fields_on_one_side:
        print("  不可比的字段", " ".join(key), field_name, side)
    return len(flips)


def positive_control_of_the_comparison(earlier_path, scratch_directory):
    print("== PCr3 ①：比对对象对它自己")
    flips_against_itself = compare(earlier_path, earlier_path)
    modified_path = os.path.join(scratch_directory, "pcr3-modified.out")
    verdict_changed = reuse_arm_changed = False
    with open(modified_path, "x", encoding="utf-8") as modified:
        for line_number, line in enumerate(open(earlier_path, encoding="utf-8"), 1):
            if (not verdict_changed) and "name=verdict" in line and " clause_S2_holds=true" in line:
                line = line.replace(" clause_S2_holds=true", " clause_S2_holds=false", 1)
                verdict_changed = True
                print(f"== 改了判决行 行{line_number}：clause_S2_holds true -> false")
            elif (not reuse_arm_changed) and "name=reuse_arm" in line and " inconsistent=0 " in line:
                line = line.replace(" inconsistent=0 ", " inconsistent=3 ", 1)
                reuse_arm_changed = True
                print(f"== 改了 reuse_arm 行 行{line_number}：inconsistent 0 -> 3")
            modified.write(line)
    print("== PCr3 ②：比对对象对改了两处的副本")
    flips_against_modified = compare(earlier_path, modified_path)
    print(f"== PCr3 结果：① 翻面 {flips_against_itself} 格（要求 0）；② 翻面 {flips_against_modified} 格（要求 2）")
    print("== PCr3", "通过" if (flips_against_itself == 0 and flips_against_modified == 2) else "不通过")


def row_one(later_path):
    stream_rows = {"first": {}, "second": {}}
    for _line_number, fields in result_rows_of(later_path):
        if fields.get("name") == "stop_clause_stream":
            print("Q1a", fields.get("stream"), "writes", fields.get("writes"), "segments", fields.get("segments"),
                  "roots", fields.get("roots"), "segment_lengths_match", fields.get("segment_lengths_match"))
        if fields.get("name") == "anchor_segment":
            stream_rows[fields["stream"]][int(fields["segment"])] = fields
    for stream, anchors, segments in (("first", ANCHOR_FIRST_STREAM_SEGMENT_STATES, range(12)),
                                      ("second", ANCHOR_SECOND_STREAM_SAMPLED_SEGMENT_STATES, SECOND_STREAM_SAMPLED_SEGMENTS)):
        unequal_segments = counted_one_by_one = formula_only_disagrees = missing = 0
        for position, segment in enumerate(segments):
            fields = stream_rows[stream].get(segment)
            if fields is None:
                missing += 1
                print("没有这一段的 anchor_segment 行", stream, segment)
                continue
            plan, crates, formula = (int(fields["counted_by_the_plan"]), int(fields["counted_by_the_crates"]), int(fields["counted_by_the_formula"]))
            numbers = [plan, crates, formula, anchors[position]]
            if len(set(numbers)) != 1:
                unequal_segments += 1
                print("不等", stream, segment, "装置计划/crates/公式/锚点", numbers)
            if plan == crates and crates != formula:
                formula_only_disagrees += 1
                print("F1（装置 = crates ≠ 公式）", stream, segment, numbers)
            if fields["in_domain_counted_one_by_one"] == "true":
                counted_one_by_one += 1
        print(stream, "段数", len(segments), "四个数不全相等的段数", unequal_segments, "逐个数 true 的段数", counted_one_by_one,
              "F1 段数", formula_only_disagrees, "缺行段数", missing)
    print("second 的 anchor_segment 行数", len(stream_rows["second"]), "three_agree 非 true 的段",
          [segment for segment, fields in sorted(stream_rows["second"].items()) if fields["three_agree"] != "true"])


def row_two(later_path):
    rows = [fields for _line_number, fields in result_rows_of(later_path)]

    def select(line_name, **wanted):
        return [fields for fields in rows if fields.get("name") == line_name and all(fields.get(field_name) == value for field_name, value in wanted.items())]

    mismatched_domain_sizes = 0
    for fields in select("anchor_stream"):
        for field_name, expected in ANCHOR_DOMAIN_SIZES[fields["stream"]].items():
            measured = int(fields[field_name])
            match_flag = fields.get(f"{field_name}_match")
            print("Q2a", fields["stream"], field_name, "量到", measured, "登记", expected, f"{field_name}_match={match_flag}")
            if measured != expected or match_flag != "true":
                mismatched_domain_sizes += 1
    print("Q2a 对不上的格数", mismatched_domain_sizes)
    without_object = []
    for cell, expected_states in CELL_STATES_EXPECTED:
        g1_rows = select("g1", cell=cell)
        has_object = len(g1_rows) == 1 and int(g1_rows[0]["states"]) == expected_states
        print("1", cell, "states", [fields["states"] for fields in g1_rows], "期望", expected_states, "有对象" if has_object else "没有对象")
        if not has_object:
            without_object.append(("1", cell))
    refeed = select("positive_control_refeed")
    refed_positions = int(refeed[0]["refed_state_unit_positions"]) if len(refeed) == 1 else 0
    print("2 refed_state_unit_positions", refed_positions)
    if refed_positions <= 0:
        without_object.append("2")
    consult = select("reuse_arm", cell="first_small", arm="k2_consult")
    consult_outcomes = int(consult[0]["distinct_outcomes"]) if len(consult) == 1 else 0
    print("3 first_small k2_consult distinct_outcomes", consult_outcomes)
    if consult_outcomes < 2:
        without_object.append("3")
    full_read_set_outcomes = {fields["cell"]: int(fields["distinct_outcomes"]) for fields in select("reuse_arm", arm="k4_full")}
    print("4 k4_full distinct_outcomes", full_read_set_outcomes)
    if not full_read_set_outcomes or max(full_read_set_outcomes.values()) < 2:
        without_object.append("4")
    timing_rows = select("positive_control_timing")
    verdict_rows = select("verdict", scope="feasibility")
    pc3_field = f"second_segment_{PC3_SEGMENT_EXPECTED}_pc3"
    pc3_field_present = len(verdict_rows) == 1 and pc3_field in verdict_rows[0]
    print("5 timing 行数", len(timing_rows), "states", sorted({fields["states"] for fields in timing_rows}),
          "pc3_segment_holds", sorted({str(fields.get("pc3_segment_holds")) for fields in timing_rows}),
          f"判决行有 {pc3_field}", pc3_field_present)
    if not (len(timing_rows) == 6 and {fields["states"] for fields in timing_rows} == {"1000"}
            and {fields.get("pc3_segment_holds") for fields in timing_rows} == {"true"} and pc3_field_present):
        without_object.append("5")
    crates_rows = select("stop_clause_crates")
    print("6a crates compared_states", [(fields["cell"], fields["compared_states"]) for fields in crates_rows])
    if not (len(crates_rows) == 3 and all(int(fields["compared_states"]) > 0 for fields in crates_rows)):
        without_object.append("6a")
    per_state_rows = select("stop_clause_crates_per_state")
    print("6b per_state states", [(fields["cell"], fields["states"]) for fields in per_state_rows])
    if not (len(per_state_rows) == 2 and all(fields["states"] == "4096" for fields in per_state_rows)):
        without_object.append("6b")
    determinism_rows = select("stop_clause_determinism")
    print("7 control_states", [(fields["cell"], fields["control_states"]) for fields in determinism_rows])
    if not (len(determinism_rows) == 2 and all(int(fields["control_states"]) > 0 for fields in determinism_rows)):
        without_object.append("7")
    removal_rows = select("positive_control_remove_one_position")
    print("8 positions", [(fields["cell"], fields["arm"], fields["positions"]) for fields in removal_rows])
    if not (removal_rows and all(int(fields["positions"]) > 0 for fields in removal_rows)):
        without_object.append("8")
    memory_rows = select("positive_control_device_memory")
    print("9 device_memory 行", [fields["cell"] for fields in memory_rows])
    if len(memory_rows) != 2:
        without_object.append("9")
    print("没有对象的格数", len(without_object), without_object)


def completeness(path):
    rows = result_rows_of(path)
    total_lines = sum(1 for _line in open(path, encoding="utf-8"))
    done_rows = [fields for _line_number, fields in rows if fields.get("name") == "done"]
    emitted = int(done_rows[-1]["emitted"]) if done_rows else None
    print("文件行数", total_lines, "E7RESULT 行数", len(rows), "收尾行 emitted", emitted, "相等" if emitted == total_lines else "不相等")
    counts = {}
    for _line_number, fields in rows:
        counts[fields.get("name")] = counts.get(fields.get("name"), 0) + 1
    for line_name in sorted(counts):
        print(f"  {counts[line_name]:4d} name={line_name}")


if __name__ == "__main__":
    subcommands = {
        "compare": lambda arguments: compare(arguments[0], arguments[1]),
        "pcr3": lambda arguments: positive_control_of_the_comparison(arguments[0], arguments[1]),
        "row-one": lambda arguments: row_one(arguments[0]),
        "row-two": lambda arguments: row_two(arguments[0]),
        "completeness": lambda arguments: completeness(arguments[0]),
    }
    subcommands[sys.argv[1]](sys.argv[2:])
