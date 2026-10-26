#!/usr/bin/env python3
# admission: always 每一次调都跑，判的是同目录 e184_r2_verdict_compare.py 此刻的样子，上一次的结论不替这一次作保
# run-condition: none 只在合成产物（内存里与临时目录）和改前产物（在工作区时）上读文本，除了 python3 之外没有环境要求，不起 cargo
"""e184_r2_verdict_compare.py 的单测（E184 重跑 r2 登记 5.2 节的对照，一条对照一个用例；登记第九节九行变异各有一项点名要红的用例）。

用法：python3 scripts/experiment-apparatus/e184_r2_verdict_compare_test.py     每项一行 ✓ 绿 / ✗ 红，末行「自证 N 项，红 M 项」，有红退 1。
"""
import contextlib
import copy
import io
import os
import sys
import tempfile

import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
from e184_r2_verdict_compare import *  # noqa: E402,F401,F403
from e184_r2_verdict_compare import Unreadable, main  # noqa: E402

# ── 合成产物：行形照改前产物（research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out）的各类行 ──────────

SMOKE_STREAMS = ("new_pool_file_creation_stream", "overwrite_fixed_script_stream", "multi_record_publish_stream", "snapshot_creation_stream")
STREAM_CHECKS = ("recorded_values_match_the_test_pins", "pins_still_in_the_test_file", "two_counting_routes_agree",
                 "two_state_not_above_full", "blocks_per_point_follow_the_local_block_length")
SCOPES = ("family:overwrite_chain_main", "family:random_history_main", "group:overwrite_chain-01_units_per_overwrite",
          "group:overwrite_chain-04_units_per_overwrite", "group:overwrite_chain-16_units_per_overwrite",
          "group:random_history-broad-FourGibibytes-steps0050", "group:random_history-broad-FourGibibytes-steps0100",
          "group:random_history-broad-FourGibibytes-steps0200", "group:random_history-broad-FourGibibytes-steps0400",
          "group:random_history-broad-UnitAreaOf240Slots-steps0100", "group:random_history-broad_with_compaction_while_mounted-FourGibibytes-steps0100")
LONG_RUN_PIECES = (["repo-reads", "selfcheck", "record-fixed-chains", "keys-fixed"]
                   + [f"{kind}-{tier}" for tier in ("random-broad-4g-s050", "random-broad-4g-s100", "random-broad-240slots-s100",
                                                    "random-compaction-4g-s100", "random-broad-4g-s200", "random-broad-4g-s400")
                      for kind in ("dump", "keys")] + ["share", "library"])
OVERLAP_WORDS = ("overlap", "partial", "separate")
SHARE_WORDS = ("independent", "renamed", "not_planned")


def stop_check_items():
    items = ["T6_no_crashflow_in_crates", "local_states_per_block_matches_crates", "local_seed_base_matches_crates",
             "local_longest_closed_form_segment_matches_crates", "copied_text_verbatim_fixed_script_head", "copied_text_verbatim_fixed_script_body",
             "copied_text_verbatim_multi_record_publish", "copied_text_verbatim_new_pool_prepare_body", "copied_segments_found"]
    return items + [f"{check}_smoke-{stream}" for stream in SMOKE_STREAMS for check in STREAM_CHECKS]


def synthetic_rows():
    """改后产物该有的样子：四类判定行、十个锚（现算的新数，条款锚除外）、作废类行、18 件、末行 name=done。每行一个字典。"""
    rows = []
    for index, item in enumerate(stop_check_items()):
        row = {"name": "stop_check", "item": item, "result": "pass"}
        if item.startswith("local_"):
            row["detail"] = f"local={65536 + index}_crates={65536 + index}"
        rows.append(row)
    for identifier, value in RECOMPUTED.items():
        clause = identifier == CLAUSE_ANCHOR
        rows.append({"name": "anchor", "identifier": identifier, "expected": "12230590578" if clause else str(value), "actual": str(value),
                     "match": "false" if clause else "true"})
    rows.append({"name": "selfcheck_done", "stop_checks": "28", "failed": "0", "anchors": "10", "anchors_mismatching": "0",
                 "clause_anchor_matches": "false", "emitted": "39"})
    for index, scope in enumerate(SCOPES):
        main_random = scope == "family:random_history_main"
        rows.append({"name": "overlap", "scope": scope, "flows": "1", "inside_fraction": "0.291667" if main_random else "0.000000",
                     "above_fraction": "0.708333" if main_random else "1.000000",
                     "at_most_smoke_interval_maximum_fraction": "0.291667" if main_random else "0.000000",
                     "verdict": "partial" if main_random else ("overlap" if index == 4 else "separate")})
        rows.append({"name": "share_group", "scope": scope, "flows": "1", "median_share_fraction": "0.000024",
                     "verdict": "not_planned" if scope.endswith("steps0400") else "independent"})
    for index in range(3):
        rows.append({"name": "control", "kind": "dump_round_trip_direct_base", "flow": f"flow{index}", "passed": "true"})
    for index in range(2):
        rows.append({"name": "determinism", "label": f"label{index}", "equal": "true"})
    rows.append({"name": "dump_done", "histories": "20", "recorded": "20", "skipped": "0", "failed": "0", "base_mismatches": "0"})
    rows.extend({"name": "pinned_smoke", "flow": f"flow{index}", "match": "true"} for index in range(2))
    for piece in LONG_RUN_PIECES:
        if piece == "library":
            rows.append({"name": "segment", "segment": piece, "state": "not_run", "reason": LIBRARY_NOT_RUN_REASON})
        else:
            rows.append({"name": "segment", "segment": piece, "state": "done", "runs": "1"})
    return rows


def render(rows, with_prefix=True, add_done=True):
    lines = [("E7RESULT " if with_prefix else "") + " ".join(f"{key}={value}" for key, value in row.items()) for row in rows]
    if add_done:
        lines.append(("E7RESULT " if with_prefix else "") + f"name=done segments=18 emitted={len(lines) + 1}")
    return "\n".join(lines) + "\n"


def varied(rows, name, key_field, key, **changes):
    """rows 的深拷贝，name 类里 key_field=key 的那一行按 changes 改字段；改到零行或多行当场报错（造副本造错了不算对照）。"""
    result = copy.deepcopy(rows)
    hit = [row for row in result if row["name"] == name and row.get(key_field) == key]
    assert len(hit) == 1, (name, key, len(hit))
    hit[0].update(changes)
    return result


def compare(before, after):
    return compare_rows(parse_rows(render(before)), parse_rows(render(after)))


def lines_named(lines, name):
    return [parse_line(line) for line in lines if line.startswith(f"E7RESULT name={name} ")]


def summaries(lines):
    return {row["reading"]: row for row in lines_named(lines, "verdict_comparison")}


def differing_cells(lines):
    return [(row["family"], row["key"]) for row in lines_named(lines, "cell") if row["same"] == "false"]


# ── 自证：每一项的标签就是登记 5.2 的对照或第九节变异表点名要红的那一格 ────────────────────

BASE = synthetic_rows()


def check_self_compare_is_not_flipped():
    lines, code = compare(BASE, BASE)
    result = summaries(lines)
    assert code == 0, code
    assert (result["narrow"]["cells"], result["narrow"]["differing"], result["narrow"]["verdict"]) == ("40", "0", "not_flipped"), result["narrow"]
    assert (result["wide"]["cells"], result["wide"]["differing"], result["wide"]["verdict"]) == ("62", "0", "not_flipped"), result["wide"]
    assert len(lines_named(lines, "cell")) == 62 and differing_cells(lines) == []
    assert [row["same"] for row in lines_named(lines, "derived")] == ["true"] * 4
    assert lines[-1] == f"E7RESULT name=done emitted={len(lines)}", lines[-1]


# 每类换一个词：(行名, 键字段, 键, 判定词字段, 改成的词, 窄读下是不是也翻)
WORD_SWAPS = (
    ("stop_check", "item", "T6_no_crashflow_in_crates", "result", "fail", True),
    ("anchor", "identifier", "snapshot_creation_two_state", "match", "false", True),
    ("selfcheck_done", "name", "selfcheck_done", "clause_anchor_matches", "true", True),
    ("overlap", "scope", "group:random_history-broad-FourGibibytes-steps0100", "verdict", "overlap", False),
    ("share_group", "scope", "group:overwrite_chain-04_units_per_overwrite", "verdict", "renamed", False),
)


def check_one_word_swapped_per_family():
    for name, key_field, key, word_field, new_word, flips_narrow in WORD_SWAPS:
        other = varied(BASE, name, key_field, key, **{word_field: new_word})
        lines, code = compare(BASE, other)
        key_in_cells = name if name == "selfcheck_done" else key
        assert differing_cells(lines) == [(name, key_in_cells)], (name, differing_cells(lines))
        assert code == 0 and summaries(lines)["wide"]["verdict"] == "flipped" and summaries(lines)["wide"]["differing"] == "1", (name, summaries(lines))
        assert (summaries(lines)["narrow"]["verdict"] == "flipped") == flips_narrow, (name, summaries(lines)["narrow"])
        assert summaries(lines)["narrow"]["cells"] == "40" and summaries(lines)["wide"]["cells"] == "62"
        cell = [row for row in lines_named(lines, "cell") if row["family"] == name and row["same"] == "false"][0]
        assert cell["before"] != cell["after"] and cell["after"] == new_word, cell


def check_numeric_fields_do_not_count():
    other = varied(BASE, "stop_check", "item", "local_states_per_block_matches_crates", detail="local=1_crates=2")
    other = varied(other, "anchor", "identifier", "snapshot_creation_two_state", actual="1")
    lines, code = compare(BASE, other)
    assert code == 0 and differing_cells(lines) == [], differing_cells(lines)
    assert summaries(lines)["narrow"]["verdict"] == "not_flipped" and summaries(lines)["wide"]["verdict"] == "not_flipped", summaries(lines)


def check_missing_and_extra_cell_report_keys_differ():
    fewer = [row for row in BASE if not (row["name"] == "anchor" and row["identifier"] == "new_pool_writes")]
    lines, code = compare(BASE, fewer)
    assert code == 5 and summaries(lines)["narrow"]["verdict"] == "keys_differ" and summaries(lines)["wide"]["verdict"] == "keys_differ", summaries(lines)
    assert differing_cells(lines) == [("anchor", "new_pool_writes")]
    assert [row for row in lines_named(lines, "cell") if row["family"] == "anchor" and row["same"] == "false"][0]["after"] == "absent"
    more = copy.deepcopy(BASE)
    more.append(copy.deepcopy([row for row in BASE if row["name"] == "overlap"][0]))
    lines, code = compare(BASE, more)
    assert code == 5 and summaries(lines)["wide"]["verdict"] == "keys_differ", summaries(lines)
    assert summaries(lines)["narrow"]["verdict"] == "not_flipped", "重复出现在宽读才有的行上，窄读不受影响"
    twice = copy.deepcopy(BASE) + [copy.deepcopy([row for row in BASE if row["name"] == "stop_check"][0])]
    lines, code = compare(twice, BASE)
    assert code == 5 and summaries(lines)["narrow"]["verdict"] == "keys_differ", "改前那份里有重复也不比"


def check_derived_clause_changes_sides():
    before_word = [row for row in BASE if row["name"] == "share_group" and row["scope"] == RANDOM_SCOPE][0]["verdict"]
    new_word = "independent" if before_word == "renamed" else "renamed"
    other = varied(BASE, "share_group", "scope", RANDOM_SCOPE, verdict=new_word)
    lines, code = compare(BASE, other)
    clauses = {row["clause"]: row for row in lines_named(lines, "derived")}
    assert (clauses["F2"]["before"], clauses["F2"]["after"], clauses["F2"]["same"]) == ("false", "true", "false"), clauses["F2"]
    assert all(clauses[each]["same"] == "true" for each in ("F1", "F3", "F6_second"))
    assert summaries(lines)["wide"]["verdict"] == "flipped" and summaries(lines)["wide"]["differing"] == "2", summaries(lines)["wide"]
    assert summaries(lines)["narrow"]["verdict"] == "not_flipped"
    chain = varied(BASE, "share_group", "scope", OVERWRITE_SCOPE, verdict="renamed")
    assert {row["clause"]: row["same"] for row in lines_named(compare(BASE, chain)[0], "derived")}["F3"] == "false"


def overlap_pair(random_fields, chain_fields):
    rows = [{"name": "overlap", "scope": RANDOM_SCOPE, "verdict": "overlap", "above_fraction": "0.100000",
             "at_most_smoke_interval_maximum_fraction": "0.100000"},
            {"name": "overlap", "scope": OVERWRITE_SCOPE, "verdict": "overlap", "above_fraction": "0.100000",
             "at_most_smoke_interval_maximum_fraction": "0.100000"}]
    rows[0].update(random_fields)
    rows[1].update(chain_fields)
    return derived_clauses(tables_of(parse_rows(render(rows, add_done=False))))


def check_second_clause_of_f6_boundary():
    assert overlap_pair({"at_most_smoke_interval_maximum_fraction": "0.500000"}, {})["F6_second"] == "true"
    assert overlap_pair({"at_most_smoke_interval_maximum_fraction": "0.499999"}, {})["F6_second"] == "false"


def check_f1_needs_both_overlap_and_both_below_half():
    assert overlap_pair({}, {})["F1"] == "true"
    assert overlap_pair({"above_fraction": "0.600000"}, {})["F1"] == "false", "above 半句不能丢"
    assert overlap_pair({}, {"above_fraction": "0.500000"})["F1"] == "false", "above 是严格小于 0.5"
    assert overlap_pair({"verdict": "partial"}, {})["F1"] == "false"
    assert overlap_pair({}, {"verdict": "separate"})["F1"] == "false"


def check_line_prefix_is_optional():
    with_prefix = parse_rows(render(BASE))
    without_prefix = parse_rows(render(BASE, with_prefix=False))
    assert len(with_prefix) == len(without_prefix) == len(BASE) + 1
    lines, code = compare_rows(with_prefix, without_prefix)
    assert code == 0 and summaries(lines)["narrow"]["verdict"] == "not_flipped" and summaries(lines)["wide"]["cells"] == "62", summaries(lines)
    assert parse_line("noise without result fields") is None and parse_line("E7RESULT progress name=stop_check") is None


def recompute_of(extra_rows):
    return {row["identifier"]: row for row in anchor_recompute_rows(tables_of(parse_rows(render(extra_rows, add_done=False))))}


def check_new_number_is_not_the_old_number():
    snapshot_full = {"name": "anchor", "identifier": "full_states_smoke-snapshot_creation_stream", "expected": "16842815", "actual": "16842815", "match": "true"}
    assert recompute_of([snapshot_full])["full_states_smoke-snapshot_creation_stream"]["recomputed"] == "16842815"
    assert recompute_of([snapshot_full])["full_states_smoke-snapshot_creation_stream"]["same"] is True
    old = dict(snapshot_full, expected="16842807", actual="16842807")
    assert recompute_of([old])["full_states_smoke-snapshot_creation_stream"]["same"] is False, "改前的数不能算同"


def check_anchor_recompute_compares_actual_and_expected():
    rows = [{"name": "anchor", "identifier": "new_pool_two_state", "expected": "16777243", "actual": "16777243", "match": "true"},
            {"name": "anchor", "identifier": "new_pool_writes", "expected": "41", "actual": "43", "match": "false"},
            {"name": "anchor", "identifier": CLAUSE_ANCHOR, "expected": "12230590578", "actual": "1358954642", "match": "false"},
            {"name": "anchor", "identifier": "first_history_seed", "expected": "7463871032432355113", "actual": "7463871032432355114", "match": "false"}]
    result = recompute_of(rows)
    assert result["new_pool_two_state"]["same"] is True
    assert result["new_pool_writes"]["same"] is False, "装置钉值 expected 不等于现算值也不算同"
    assert result[CLAUSE_ANCHOR]["same"] is True, "条款锚只比 actual，expected 是条款写的数"
    assert result["first_history_seed"]["same"] is False
    assert result["snapshot_creation_two_state"]["actual"] == "absent" and result["snapshot_creation_two_state"]["same"] is False, "产物里没有的锚不算同"
    assert len(result) == 10


def check_closed_form_gives_old_numbers_on_the_old_shape():
    assert (states_of(SNAPSHOT_SEGMENTS_OLD_SHAPE, segment_two_state), states_of(SNAPSHOT_SEGMENTS_OLD_SHAPE, segment_full)) == (16842782, 16842807)
    assert states_of(NEW_POOL_SEGMENTS_OLD_SHAPE, segment_two_state) == 16777240 and states_of(NEW_POOL_SEGMENTS_OLD_SHAPE, segment_full) == 16777260
    assert states_of(NEW_POOL_SEGMENTS_OLD_SHAPE, segment_quick) == 46 and writes_of(NEW_POOL_SEGMENTS_OLD_SHAPE) == 41
    assert (states_of(MULTI_RECORD_SEGMENTS_OLD_SHAPE, segment_two_state), states_of(MULTI_RECORD_SEGMENTS_OLD_SHAPE, segment_full)) == (1358954604, 1358954634)
    assert (len(SNAPSHOT_SEGMENTS_OLD_SHAPE), writes_of(SNAPSHOT_SEGMENTS_OLD_SHAPE)) == (15, 62)


def check_closed_form_gives_new_numbers_and_the_increment_route_agrees():
    assert (len(SNAPSHOT_SEGMENTS), writes_of(SNAPSHOT_SEGMENTS)) == (16, 64) and (len(MULTI_RECORD_SEGMENTS), writes_of(MULTI_RECORD_SEGMENTS)) == (20, 117)
    assert (states_of(SNAPSHOT_SEGMENTS, segment_two_state), states_of(SNAPSHOT_SEGMENTS, segment_full)) == (16842785, 16842815)
    assert states_of(SNAPSHOT_SEGMENTS, segment_quick) == 67
    assert (states_of(NEW_POOL_SEGMENTS, segment_two_state), states_of(NEW_POOL_SEGMENTS, segment_full), states_of(NEW_POOL_SEGMENTS, segment_quick)) == (16777243, 16777268, 54)
    assert (len(NEW_POOL_SEGMENTS), writes_of(NEW_POOL_SEGMENTS)) == (12, 43)
    assert (states_of(MULTI_RECORD_SEGMENTS, segment_two_state), states_of(MULTI_RECORD_SEGMENTS, segment_full)) == (1358954607, 1358954642)
    # 增量路：多一个系统配置槽 2 写段，两态 +3、全量 +8
    assert (16842782 + 3, 16842807 + 8) == (16842785, 16842815) and (16777240 + 3, 16777260 + 8, 46 + 8) == (16777243, 16777268, 54)
    assert (1358954604 + 3, 1358954634 + 8) == (1358954607, 1358954642)
    assert FIXED_SCRIPT_NEW == {"two_state": 1678377095, "full": 1678377220, "quick": 294, "segments": 80}, FIXED_SCRIPT_NEW
    assert FIXED_SCRIPT_NEW["two_state"] + FIXED_SCRIPT_CONFIG_SLOT_SEGMENTS_NEW * CONFIG_SLOT_FULL_MINUS_TWO_STATE == FIXED_SCRIPT_NEW["full"]
    assert FIXED_SCRIPT_OLD["two_state"] + FIXED_SCRIPT_CONFIG_SLOT_SEGMENTS_OLD * CONFIG_SLOT_FULL_MINUS_TWO_STATE == FIXED_SCRIPT_OLD["full"]
    assert RECOMPUTED["blocks_needed_at_and_one_state_beyond_the_states_per_block"] == "1,2"


def check_void_check_counts_failures():
    rows = copy.deepcopy(BASE)
    assert {row["family"]: (row["cells"], row["failing"]) for row in void_check_rows(parse_rows(render(rows)))} == {
        "control": (3, 0), "determinism": (2, 0), "dump_done": (1, 0), "pinned_smoke": (2, 0)}
    rows = varied(rows, "determinism", "label", "label1", equal="false")
    rows = varied(rows, "dump_done", "histories", "20", base_mismatches="2")
    rows = varied(rows, "pinned_smoke", "flow", "flow0", match="false")
    rows = varied(rows, "control", "flow", "flow2", passed="false")
    counted = {row["family"]: row["failing"] for row in void_check_rows(parse_rows(render(rows)))}
    assert counted == {"control": 1, "determinism": 1, "dump_done": 1, "pinned_smoke": 1}, counted
    lines, _ = compare(BASE, rows)
    assert {row["family"]: row["failing"] for row in lines_named(lines, "void_check")} == {key: str(value) for key, value in counted.items()}


def check_incomplete_product_is_refused():
    assert completeness_problems(parse_rows(render(BASE))) == []
    assert completeness_problems(parse_rows(render(BASE, add_done=False)))
    text = render(BASE)
    wrong = text[:text.rindex("emitted=")] + "emitted=9" + text[text.rindex("emitted=") + len("emitted="):]
    assert wrong != text and completeness_problems(parse_rows(wrong))
    running = varied(BASE, "segment", "segment", "share", state="running")
    assert any("share" in problem for problem in completeness_problems(parse_rows(render(running))))
    missing_piece = [row for row in BASE if not (row["name"] == "segment" and row["segment"] == "keys-fixed")]
    assert completeness_problems(parse_rows(render(missing_piece)))
    library_ran = varied(BASE, "segment", "segment", "library", state="done")
    assert completeness_problems(parse_rows(render(library_ran))) == []
    with tempfile.TemporaryDirectory(prefix="e184-r2-compare-test-") as directory:
        path = os.path.join(directory, "cut.out")
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(render(BASE, add_done=False))
        try:
            load_product(path)
        except Unreadable:
            return
    raise AssertionError("没写完的产物应当拒")


def check_end_to_end_through_files():
    with tempfile.TemporaryDirectory(prefix="e184-r2-compare-test-") as directory:
        before, after = os.path.join(directory, "before.out"), os.path.join(directory, "after.out")
        with open(before, "w", encoding="utf-8") as handle:
            handle.write(render(BASE))
        with open(after, "w", encoding="utf-8") as handle:
            handle.write(render(varied(BASE, "anchor", "identifier", "new_pool_writes", match="false")))
        captured = io.StringIO()
        with contextlib.redirect_stdout(captured):
            code = main([before, after])
        assert code == 0
        text = captured.getvalue().splitlines()
        assert text[-1] == f"E7RESULT name=done emitted={len(text)}" and len(text) == 62 + 4 + 10 + 4 + 2 + 1, len(text)
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            assert main([before]) == 2 and main([before, os.path.join(directory, "missing.out")]) == 2


BEFORE_PRODUCT = "research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out"
BEFORE_PRODUCT_SHA256 = "7eacaffaddd44d34bf4a67afe29214ccbbf7ffbd63663d61ef7489124e858c06"


def check_real_before_product_when_present():
    path = os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..", BEFORE_PRODUCT)
    if not os.path.exists(path):
        print("    （改前产物不在工作区，这一项跳过）")
        return
    import hashlib
    with open(path, "rb") as handle:
        assert hashlib.sha256(handle.read()).hexdigest() == BEFORE_PRODUCT_SHA256, "改前产物的 sha256 变了"
    rows = load_product(path)
    assert len(rows) == 950
    lines, code = compare_rows(rows, rows)
    result = summaries(lines)
    assert code == 0 and (result["narrow"]["cells"], result["narrow"]["verdict"]) == ("40", "not_flipped"), result
    assert (result["wide"]["cells"], result["wide"]["differing"], result["wide"]["verdict"]) == ("62", "0", "not_flipped"), result
    assert {row["family"]: row["cells"] for row in lines_named(lines, "void_check")} == {"control": "174", "determinism": "12", "dump_done": "6", "pinned_smoke": "10"}


CHECKS = (
    ("自比：窄读 40 格、宽读 62 格与四条条款全部相同，判不翻（登记 5.2 自比；变异 M1、M7）", check_self_compare_is_not_flipped),
    ("每类换一个词：恰好那一格不同、判翻并点名（登记 5.2；变异 M1、M7）", check_one_word_swapped_per_family),
    ("只改数值字段不翻（登记 5.2；变异 M2）", check_numeric_fields_do_not_count),
    ("少一格、多一格报键不齐（登记 5.2；变异 M3、M4）", check_missing_and_extra_cell_report_keys_differ),
    ("推出的条款换边（登记 5.2）", check_derived_clause_changes_sides),
    ("F6 第二条的边界 0.500000 与 0.499999（登记 5.2；变异 M5）", check_second_clause_of_f6_boundary),
    ("F1 要两行都是 overlap 且 above_fraction 都小于 0.5（变异 M6）", check_f1_needs_both_overlap_and_both_below_half),
    ("行首 E7RESULT 有没有都认（登记 5.2；变异 M8）", check_line_prefix_is_optional),
    ("建快照流全量的现算值是 16842815，不是旧数（变异 M9）", check_new_number_is_not_the_old_number),
    ("anchor_recompute 比 actual 与 expected，条款锚只比 actual", check_anchor_recompute_compares_actual_and_expected),
    ("整式路在旧段形上算回登记写死的旧数（登记 5.2 现算路的对照）", check_closed_form_gives_old_numbers_on_the_old_shape),
    ("整式路在新段形上的数与增量路相同", check_closed_form_gives_new_numbers_and_the_increment_route_agrees),
    ("作废类行的计数", check_void_check_counts_failures),
    ("没写完的产物不比（登记 XR4）", check_incomplete_product_is_refused),
    ("经文件端到端，用法错与读不了退 2", check_end_to_end_through_files),
    ("改前产物在时：sha256、950 行、自比 40 与 62 格", check_real_before_product_when_present),
)


def run_checks():
    red = 0
    for label, check in CHECKS:
        try:
            check()
        except Exception as error:  # 任何异常都算这一项红
            red += 1
            print(f"  ✗ 红：{label}（{type(error).__name__}：{str(error)[:200]}）")
            print("     → 怎么办：按这一项的名字找 e184_r2_verdict_compare.py 里对应的判法改，改完重跑这份自证")
        else:
            print(f"  ✓ 绿：{label}")
    print(f"自证 {len(CHECKS)} 项，红 {red} 项")
    return 1 if red else 0


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(run_checks())
