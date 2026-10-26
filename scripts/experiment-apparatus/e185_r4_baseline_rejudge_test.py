#!/usr/bin/env python3
# admission: always 每一次调都跑，判的是同目录 e185_r4_baseline_rejudge.py 此刻的样子，上一次的结论不替这一次作保
# run-condition: none 只在合成产物（临时目录）和三份已有产物（在工作区时）上读文本，除了 python3 之外没有环境要求，不起 cargo
"""e185_r4_baseline_rejudge.py 的单测（E185 第 4 次重跑登记第九节重判脚本那几行变异各有一项点名要红的自证，标签即变异表的名字前缀）。

用法：python3 scripts/experiment-apparatus/e185_r4_baseline_rejudge_test.py     每项一行 ✓ 绿 / ✗ 红，末行「自证 N 项，红 M 项」，有红退 1。
"""
import hashlib
import os
import sys
import tempfile

import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
import e185_r4_baseline_rejudge as rejudge_module  # noqa: E402
from e185_r4_baseline_rejudge import *  # noqa: E402,F401,F403

# ── 合成产物（字段的写法照装置的产物行） ────────────────────

POINT = ("0.25", "1024", "4")


def stage_row(round_number, call_index, kind, crc, scan_increment, choose, release):
    """一条 name=stage 行（只带门槛账要的那几栏）。"""
    return {"name": "stage", "fraction": POINT[0], "pool_data_units": POINT[1], "batch_units": POINT[2], "round": str(round_number),
            "call_index": str(call_index), "kind": kind, "crc_cpu_nanoseconds": f"{crc:.6f}", "scan_increment_cpu_nanoseconds": f"{scan_increment:.6f}",
            "choose_cpu_nanoseconds": "none" if choose is None else f"{choose:.6f}", "release_check_crc_cpu_nanoseconds": f"{release:.6f}"}


def stage_rows(issue_crc=100_000.0, batch_crc=300_000.0, scan_increment=50_000.0, choose=200_000.0, release=280_000.0, issue_release=0.0):
    """5 轮，每轮签发一次加两批：A 省 100 + 300 + 300 = 700 微秒、提交 3 次（0.233333 ms）；B 加签发的选段 200（0.3 ms）；
    C 是两批各 300 + 50 加签发 100 + 50 + 200（1050 微秒，0.35 ms）；D 省两批各 280（0.28 ms，提交 2 次）。"""
    rows = []
    for round_number in range(1, ROUNDS + 1):
        rows.append(stage_row(round_number, 0, "issue", issue_crc, scan_increment, choose, issue_release))
        rows.append(stage_row(round_number, 1, "batch", batch_crc, scan_increment, None, release))
        rows.append(stage_row(round_number, 2, "final_batch", batch_crc, scan_increment, None, release))
    return rows


def writing_stage_row(round_number, call_index, kind, crc_today, scan_today, lookup, build, single, three_way, release_today, release_single,
                      release_three_way):
    return {"name": "cpu_writing_stage", "fraction": POINT[0], "pool_data_units": POINT[1], "batch_units": POINT[2], "round": str(round_number),
            "call_index": str(call_index), "kind": kind, "crc_today_cpu_nanoseconds": f"{crc_today:.6f}",
            "crc_hardware_single_cpu_nanoseconds": f"{single:.6f}", "crc_hardware_three_way_cpu_nanoseconds": f"{three_way:.6f}",
            "release_check_crc_today_cpu_nanoseconds": f"{release_today:.6f}", "release_check_crc_hardware_single_cpu_nanoseconds": f"{release_single:.6f}",
            "release_check_crc_hardware_three_way_cpu_nanoseconds": f"{release_three_way:.6f}", "scan_today_cpu_nanoseconds": f"{scan_today:.6f}",
            "scan_identity_index_lookup_cpu_nanoseconds": f"{lookup:.6f}", "identity_index_build_cpu_nanoseconds": f"{build:.6f}",
            "readable": "true"}


def writings_product_rows(factor=0.5, factor_status="measured", today_factor=1.0, baseline="4000000.0"):
    """这一趟写法与拆分段：同一套 stage 行，每次调用的九样计时件（今天的 CRC 100 或 300 微秒、扫描 = CRC + 50、查索引 = CRC + 10、建索引 20、
    硬件单路是今天的 1/4、三路 1/8），每种写法每样阶段的 φ 都写 factor（写法甲写 today_factor）。"""
    rows = [{"name": "calibration_baseline", "segment": "cpu_writings", "position": "start", "median_nanoseconds": baseline}]
    for row in stage_rows():
        rows.append(row)
        crc = float(row["crc_cpu_nanoseconds"])
        release = float(row["release_check_crc_cpu_nanoseconds"])
        rows.append(writing_stage_row(row["round"], row["call_index"], row["kind"], crc, crc + 50_000.0, crc + 10_000.0, 20_000.0, crc / 4, crc / 8,
                                      release, release / 4, release / 8))
    for writing in WRITINGS:
        for stage in STAGES:
            value = today_factor if writing == TODAY else factor
            rows.append({"name": "writing_factor", "fraction": POINT[0], "pool_data_units": POINT[1], "batch_units": POINT[2], "writing": writing,
                         "stage": stage, "factor_per_round": ",".join([f"{value:.6f}"] * ROUNDS), "factor": f"{value:.6f}",
                         "status": "measured" if writing == TODAY else factor_status})
    return rows


def point_line(candidate, threshold, values, floor_values, judgement_per_round, judgement, floor_per_round, floor_judgement,
               kind="attack_ledger", clock="thread", form="shared", index="0", name="host_cpu_per_submission_point", arm=None):
    row = {"name": name}
    if arm is not None:
        row["arm"] = arm
    row.update({"fraction": POINT[0], "pool_data_units": POINT[1], "batch_units": POINT[2], "candidate": candidate, "index": index,
                "form": form, "wait": "blocking", "gap_milliseconds": "0", "clock": clock, "threshold_kind": kind,
                "threshold_milliseconds": threshold, "submission_units": "8,8,4",
                "host_cpu_per_submission_milliseconds_per_round": values, "judgement_per_round": judgement_per_round, "judgement": judgement,
                "floor_host_cpu_per_submission_milliseconds_per_round": floor_values, "floor_judgement_per_round": floor_per_round,
                "floor_judgement": floor_judgement})
    return row


def render(rows):
    return "".join("E7RESULT " + " ".join(f"{key}={value}" for key, value in row.items()) + "\n" for row in rows)


def ledger_text(break_even="0.3500"):
    return (f"name=g1_ledger f={POINT[0]} P={POINT[1]} K={POINT[2]} saved_upper_ms=0.001 submissions_per_intent=3 "
            f"break_even_host_cpu_per_submission_ms={break_even}\n")


def r2_ledger_rows(**kwargs):
    return ledger_rows_of_the_stage_lines(stage_rows(**kwargs))


def factors_of(rows):
    return writing_factors_of(rows)


# ── 自证：每一项的标签是变异表那一行的名字前缀 ────────────────────

def check_equal_threshold_is_losing():
    assert judgement_against_the_threshold(0.3, 0.3, "scan_checksum_only") == "losing"
    assert judgement_against_the_threshold(0.299999, 0.3, "scan_checksum_only") == "not_losing"


def check_rejudge_reads_the_crt_column_not_the_floor():
    line = point_line("scan_checksum_only", "0.300000", "0.1,0.1,0.1,0.1,0.1", "0.5,0.5,0.5,0.5,0.5", "", "", "", "")
    rejudged = rejudge_the_line(line, 0.3)
    assert rejudged["judgement"] == "not_losing", rejudged
    assert rejudged["floor_judgement"] == "losing", rejudged


def check_machine_ratio_is_local_over_peer():
    assert machine_ratio(1_000_000.0, 2_000_000.0) == 0.5
    assert machine_ratio(None, 2.0) is None


def rho_of_every_source(value):
    return {(crt, source): value for crt in (CRT_FROM_R3, CRT_FROM_THE_DRIFT_CONTROL) for source in THRESHOLD_SOURCES}


def check_machine_scaled_multiplies_rho_once():
    line = point_line("scan_checksum_only", "0.400000", "0.1,0.1,0.1,0.1,0.1", "0.05,0.05,0.05,0.05,0.05", "", "not_losing", "", "not_losing",
                      kind="machine_scaled")
    base = {(TODAY, source, POINT, "scan_checksum_only"): 0.25 for source in THRESHOLD_SOURCES}
    rows = rejudged_rows([line], None, base, rho_of_every_source(2.0), set())
    assert {row["threshold"] for row in rows if row["writing"] == TODAY} == {0.5}, rows[:2]


def check_choice_candidate_is_a_lower_bound():
    assert judgement_against_the_threshold(0.1, 0.3, "scan_checksum_and_choice") == "not_losing_lower_bound"
    assert judgement_against_the_threshold(0.1, 0.3, "every_stage") == "not_losing"


def check_negative_scan_increment_is_not_run():
    thresholds = thresholds_of_the_ledger(r2_ledger_rows(scan_increment=-1_000.0))
    every_stage = thresholds[(POINT, "every_stage")]
    assert every_stage["status"] == "not_run" and every_stage["reason"] == "scan_increment_below_zero", every_stage
    assert thresholds[(POINT, "scan_checksum_only")]["status"] == "measured"


def check_drift_crt_source_reads_the_controlled_arm():
    reproduction = point_line("scan_checksum_only", "0.3", "1,1,1,1,1", "1,1,1,1,1", "", "losing", "", "losing", name="drift_control_point",
                              arm="reproduction")
    controlled = point_line("scan_checksum_only", "0.3", "2,2,2,2,2", "2,2,2,2,2", "", "losing", "", "losing", name="drift_control_point",
                            arm="controlled")
    picked = point_lines_of_the_drift_control_product([reproduction, controlled])
    assert picked == [controlled], picked


def check_changed_compares_the_judgement_not_the_floor():
    line = point_line("scan_checksum_only", "0.300000", "0.5,0.5,0.5,0.5,0.5", "0.1,0.1,0.1,0.1,0.1",
                      "losing,losing,losing,losing,losing", "losing", "not_losing,not_losing,not_losing,not_losing,not_losing", "not_losing")
    base = {(TODAY, source, POINT, "scan_checksum_only"): 0.3 for source in THRESHOLD_SOURCES}
    rows = [row for row in rejudged_rows([line], None, base, rho_of_every_source(1.0), set()) if row["writing"] == TODAY]
    assert all(row["changed"] == "false" and row["floor_changed"] == "false" for row in rows), rows


def check_factor_does_not_scale_the_choice():
    rows = scaled_rows_of_the_writing(r2_ledger_rows(), factors_of(writings_product_rows(factor=0.5)), "hardware_crc_single")
    thresholds = thresholds_of_the_ledger(rows)
    saved_a = round(thresholds[(POINT, "scan_checksum_only")]["saved_ms"] * 1e6)
    saved_b = round(thresholds[(POINT, "scan_checksum_and_choice")]["saved_ms"] * 1e6)
    assert saved_a == 350_000, saved_a
    assert saved_b == saved_a + 200_000, (saved_a, saved_b)


def check_today_factor_is_one():
    r2_rows = r2_ledger_rows()
    rows = scaled_rows_of_the_writing(r2_rows, factors_of(writings_product_rows(today_factor=1.1)), TODAY)
    assert rows == r2_rows, rows[:2]


def check_release_check_excludes_the_issue():
    thresholds = thresholds_of_the_ledger(r2_ledger_rows(issue_release=50_000.0))
    release = thresholds[(POINT, "release_check_only")]
    assert round(release["saved_ms"] * 1e6) == 560_000 and release["submissions"] == 2, release


def check_thread_line_count_is_checked():
    line = point_line("scan_checksum_only", "none", "0.1,0.1,0.1,0.1,0.1", "0.1,0.1,0.1,0.1,0.1", "not_run,not_run,not_run,not_run,not_run",
                      "not_run", "not_run,not_run,not_run,not_run,not_run", "not_run")
    assert reproduction_findings([line], {}, None, expected_thread_lines=1) == []
    findings = reproduction_findings([line], {}, None, expected_thread_lines=2)
    assert findings == ["clause_k_s3_thread_clock_point_lines_1_not_2"], findings


def check_today_over_r2_band_edges_are_inclusive():
    r4_rows = r2_ledger_rows(issue_crc=80_000.0, batch_crc=240_000.0, scan_increment=40_000.0, release=224_000.0)
    lines, outside = today_over_r2_lines(r4_rows, r2_ledger_rows())
    assert outside == 0 and all(line.endswith("judgement=inside_the_band") for line in lines), lines
    assert any("stage=crc ratio=0.800000" in line for line in lines), lines


def knob_row(writing, judgement, key=("0.25", "1024", "4", "scan_checksum_only", "0", "shared", "blocking", "0", "thread", "attack_ledger")):
    return {"writing": writing, "threshold_source": RATIO_ON_THE_R2_PRODUCT, "crt_source": CRT_FROM_R3, "key": key, "judgement": judgement}


def check_knob_flip_needs_a_judgement_at_both_ends():
    unjudged = knob_flip_counts([knob_row("hardware_crc_single", "losing"), knob_row("hardware_crc_three_way", "rounds_insufficient")])
    assert [entry for entry in unjudged if entry[0] == "hardware_crc_lanes"] == [], unjudged
    flipped = knob_flip_counts([knob_row("hardware_crc_single", "losing"), knob_row("hardware_crc_three_way", "not_losing")])
    lanes = [entry for entry in flipped if entry[0] == "hardware_crc_lanes"]
    assert lanes == [("hardware_crc_lanes", "hardware_crc_single", "hardware_crc_three_way", "scan_checksum_only", 1, 1)], lanes


def check_voided_writing_rejudges_to_void():
    line = point_line("scan_checksum_only", "0.3", "0.1,0.1,0.1,0.1,0.1", "0.1,0.1,0.1,0.1,0.1", "", "not_losing", "", "not_losing")
    base = {(writing, source, POINT, "scan_checksum_only"): 0.3 for writing in WRITINGS for source in THRESHOLD_SOURCES}
    rows = rejudged_rows([line], None, base, rho_of_every_source(1.0), {"hardware_crc_single"})
    judgements = {row["writing"]: row["judgement"] for row in rows}
    assert judgements["hardware_crc_single"] == "void_positive_control", judgements
    assert judgements[TODAY] == "not_losing" and judgements["hardware_crc_three_way"] == "not_losing", judgements


def check_direct_source_takes_the_column_of_the_writing():
    writing_rows = writings_product_rows()
    stage = ledger_rows_of_the_stage_lines(writing_rows)
    single = thresholds_of_the_ledger(direct_rows_of_the_writing(stage, writing_rows, "hardware_crc_single"))
    three_way = thresholds_of_the_ledger(direct_rows_of_the_writing(stage, writing_rows, "hardware_crc_three_way"))
    with_build = thresholds_of_the_ledger(direct_rows_of_the_writing(stage, writing_rows, "identity_index_with_build"))
    assert round(single[(POINT, "scan_checksum_only")]["saved_ms"] * 1e6) == 175_000, single
    assert round(three_way[(POINT, "scan_checksum_only")]["saved_ms"] * 1e6) == 87_500, three_way
    # 计入建索引：乙段增量 = 查（CRC + 10）+ 建（20）− 今天的 CRC = 30 微秒；C 省 3 × (CRC 之和) + 3 × 30 + 选段 200
    assert round(with_build[(POINT, "every_stage")]["saved_ms"] * 1e6) == 700_000 + 90_000 + 200_000, with_build


def r3_product_rows(judgement="not_losing", printed_ratio="2.000000"):
    """合成的 r3 第一段：A 两种门槛的机器各一行（ρ = 6 ÷ 3 = 2）、C 一行用 g1 的数；c_rt 0.1、c_floor 0.05。"""
    rows = [{"name": "calibration_baseline", "segment": "host_cpu_per_submission", "position": "start", "median_nanoseconds": "6000000.0"},
            {"name": "machine_ratio", "ratio": printed_ratio}]
    for candidate, kind, threshold in (("scan_checksum_only", "attack_ledger", "0.233333"), ("scan_checksum_only", "machine_scaled", "0.466667"),
                                       ("every_stage", "attack_ledger", "0.350000")):
        rows.append(point_line(candidate, threshold, "0.1,0.1,0.1,0.1,0.1", "0.05,0.05,0.05,0.05,0.05", ",".join([judgement] * ROUNDS), judgement,
                               ",".join(["not_losing"] * ROUNDS), "not_losing", kind=kind))
    return rows


def write_the_synthetic_inputs(directory, judgement="not_losing", printed_ratio="2.000000"):
    r2 = [{"name": "calibration_baseline", "segment": "stage_split", "position": "start", "median_nanoseconds": "3000000.0"}] + stage_rows()
    paths = {}
    for item, text in (("host_cpu_product", render(r3_product_rows(judgement, printed_ratio))), ("stage_split_product", render(r2)),
                       ("writings_product", render(writings_product_rows())), ("attack_ledger", ledger_text())):
        paths[item] = os.path.join(directory, item)
        with open(paths[item], "w", encoding="utf-8") as handle:
            handle.write(text)
    return paths


def run_on_the_synthetic_inputs(judgement="not_losing", break_one_sha256=False, printed_ratio="2.000000"):
    """打一遍合成的五份输入（只有四份：控漂移段不给），把登记的 sha256 与线程钟行数临时换成合成的那几个；交回（退出码、行）。"""
    saved = (rejudge_module.REGISTERED_SHA256, rejudge_module.THREAD_CLOCK_POINT_LINES_OF_THE_R3_PRODUCT)
    with tempfile.TemporaryDirectory() as directory:
        paths = write_the_synthetic_inputs(directory, judgement, printed_ratio)
        registered = {}
        for item in ("attack_ledger", "stage_split_product", "host_cpu_product"):
            with open(paths[item], "rb") as handle:
                registered[item] = hashlib.sha256(handle.read()).hexdigest()
        if break_one_sha256:
            registered["stage_split_product"] = "0" * 64
        rejudge_module.REGISTERED_SHA256 = registered
        rejudge_module.THREAD_CLOCK_POINT_LINES_OF_THE_R3_PRODUCT = 3
        try:
            out = []
            arguments = []
            for item in ("host_cpu_product", "stage_split_product", "writings_product", "attack_ledger"):
                arguments += ["--" + item.replace("_", "-"), paths[item]]
            code = run(arguments, out)
        finally:
            rejudge_module.REGISTERED_SHA256, rejudge_module.THREAD_CLOCK_POINT_LINES_OF_THE_R3_PRODUCT = saved
    return code, out


def check_reproduction_end_to_end_on_synthetic_products():
    code, out = run_on_the_synthetic_inputs()
    text = "\n".join(out)
    assert code == 0, text[-2000:]
    assert "name=positive_control row=reproduction_of_the_r3_product lines=3 findings=0 holds=true" in text, text[:2000]
    assert sum(1 for line in out if line.startswith("name=baseline_rejudge_point ")) == 3 * len(WRITINGS) * len(THRESHOLD_SOURCES)
    assert "name=baseline_rejudge_point writing=hardware_crc_single threshold_source=ratio_on_the_r2_product crt_source=r3_host_cpu_per_submission " in text
    code, out = run_on_the_synthetic_inputs(judgement="losing")
    assert code == 3 and any(line.startswith("name=stop_clause clause=K-S3 ") for line in out), out


def check_input_sha256_mismatch_stops():
    code, out = run_on_the_synthetic_inputs(break_one_sha256=True)
    assert code == 3, out
    assert any(line.startswith("name=stop_clause clause=K-S5 finding=clause_k_s5_stage_split_product_sha256_") for line in out), out
    assert not any(line.startswith("name=baseline_rejudge_point ") for line in out)


def check_unstable_count_takes_the_judgement_column_of_each_card():
    lines = [point_line("scan_checksum_only", "0.3", "", "", "", judgement, "", "not_applicable", index=index)
             for index, judgement in (("0", "unstable"), ("0", "unstable"), ("0", "not_run"), ("1", "unstable"), ("1", "losing"))]
    assert unstable_count_lines(lines) == ["name=unstable_count_r3 index=0 lines=3 unstable=2",
                                           "name=unstable_count_r3 index=1 lines=2 unstable=1"]


def check_failure_clause_counts_only_not_losing_turned_losing():
    rows = [{"writing": "hardware_crc_single", "threshold_source": RATIO_ON_THE_R2_PRODUCT, "crt_source": CRT_FROM_R3, "changed": changed,
             "judgement": judgement, "r3_judgement": r3_judgement}
            for changed, judgement, r3_judgement in (("true", "losing", "not_losing"), ("true", "losing", "unstable"), ("false", "losing", "losing"))]
    lines = failure_clause_lines(rows, [], 0)
    assert lines[0] == ("name=failure_clause clause=L1 writing=hardware_crc_single threshold_source=ratio_on_the_r2_product "
                        "crt_source=r3_host_cpu_per_submission changed_to_losing=1 triggered=true"), lines


def check_summary_counts_changed_to_losing_per_writing_and_card():
    rows = [{"writing": "hardware_crc_single", "threshold_source": RATIO_ON_THE_R2_PRODUCT, "crt_source": CRT_FROM_R3,
             "candidate": "scan_checksum_only", "index": "0", "threshold_kind": "machine_scaled", "clock": "thread",
             "fraction": POINT[0], "pool_data_units": POINT[1], "batch_units": POINT[2], "changed": changed, "judgement": judgement}
            for changed, judgement in (("true", "losing"), ("false", "losing"), ("true", "unstable"), ("false", "not_losing"))]
    lines = summary_lines(rows)
    summary = next(line for line in lines if line.startswith("name=baseline_rejudge_summary "))
    assert " lines=4 not_losing=1 not_losing_lower_bound=0 losing=2 unstable=1 " in summary, summary
    assert " changed=2 changed_to_losing=1 changed_to_unstable=1 changed_to_not_losing=0 " in summary, summary
    point_summary = next(line for line in lines if line.startswith("name=baseline_rejudge_point_summary "))
    assert point_summary.endswith("forms_not_losing=1 forms_losing=2 forms_unstable=1 forms_judged=4"), point_summary


def check_machine_ratio_differing_from_the_r3_product_stops():
    code, out = run_on_the_synthetic_inputs(printed_ratio="2.000001")
    assert code == 3, out
    assert any(line.startswith("name=stop_clause clause=K-S4 finding=clause_k_s4_machine_ratio_2.000000_differs_from_the_r3_product_2.000001")
               for line in out), out


REAL_PRODUCTS = {
    "host_cpu_product": "research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out",
    "stage_split_product": "research/results/e185-gc-batch-wall-clock-split-stage-split-r2-2026-10-23.out",
    "attack_ledger": "research/prompts/m4-gpugc-design-r1-opus-model/out/g1.out",
}


def check_real_products_reproduce_the_r3_product_when_present():
    root = os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..")
    paths = {item: os.path.join(root, path) for item, path in REAL_PRODUCTS.items()}
    if not all(os.path.exists(path) for path in paths.values()):
        print("    （三份已有产物不全在工作区，这一项跳过）")
        return
    texts = {}
    for item, path in paths.items():
        with open(path, "rb") as handle:
            data = handle.read()
        assert hashlib.sha256(data).hexdigest() == REGISTERED_SHA256[item], item
        texts[item] = data.decode("utf-8")
    r3_lines, r2_lines = lines_of(texts["host_cpu_product"]), lines_of(texts["stage_split_product"])
    attack_points = attack_ledger_points_of(texts["attack_ledger"])
    thresholds = thresholds_of_the_ledger(ledger_rows_of_the_stage_lines(r2_lines))
    base = {(point, candidate): (attack_points.get(point) if candidate == EVERY_STAGE else entry["threshold"])
            for (point, candidate), entry in thresholds.items()}
    rho = machine_ratio(calibration_baseline_of(r3_lines, "host_cpu_per_submission"), calibration_baseline_of(r2_lines, "stage_split"))
    point_lines = point_lines_of_the_r3_product(r3_lines)
    assert len(point_lines) == 2684, len(point_lines)
    assert reproduction_findings(point_lines, base, rho) == []


CHECKS = (
    ("R4J1 重判的等号在 losing 那一侧", check_equal_threshold_is_losing),
    ("R4J2 重判读 c_rt 那一列、不读 c_floor", check_rejudge_reads_the_crt_column_not_the_floor),
    ("R4J3 ρ 是本机 ÷ 量阶段 CPU 的那一台", check_machine_ratio_is_local_over_peer),
    ("R4J4 换到本机的门槛只乘一次 ρ", check_machine_scaled_multiplies_rho_once),
    ("R4J5 候选 B 的不赔标下界", check_choice_candidate_is_a_lower_bound),
    ("R4J6 乙段增量为负的点 C 写 not_run", check_negative_scan_increment_is_not_run),
    ("R4J7 c_rt 来源读控漂移段受控臂", check_drift_crt_source_reads_the_controlled_arm),
    ("R4J8 变没变比的是 judgement", check_changed_compares_the_judgement_not_the_floor),
    ("R4J9 输入 sha256 对不上就停（K-S5）", check_input_sha256_mismatch_stops),
    ("R4J10 线程钟那一半的行数要对（K-S3）", check_thread_line_count_is_checked),
    ("R4J11 B7 的两条边含在带里", check_today_over_r2_band_edges_are_inclusive),
    ("R4J12 旋钮翻面两端都要有判定", check_knob_flip_needs_a_judgement_at_both_ends),
    ("R4J13 阳性对照红的写法整段作废", check_voided_writing_rejudges_to_void),
    ("R4J14 直接算那一端取写法 X 那一列", check_direct_source_takes_the_column_of_the_writing),
    ("R4J15 合成产物上逐行复现、改一行判定就停（PR1、K-S3）", check_reproduction_end_to_end_on_synthetic_products),
    ("R4J16 D5 数的是每张卡 judgement 列的 unstable", check_unstable_count_takes_the_judgement_column_of_each_card),
    ("R4J17 L1 只数从不赔变赔的", check_failure_clause_counts_only_not_losing_turned_losing),
    ("R4J18 B5、B6 按写法与卡数变成赔与不稳定的", check_summary_counts_changed_to_losing_per_writing_and_card),
    ("R4J19 ρ 与 r3 打出的不同就停（K-S4）", check_machine_ratio_differing_from_the_r3_product_stops),
    ("V166 系数不乘选段", check_factor_does_not_scale_the_choice),
    ("V167 写法甲的系数恒为 1", check_today_factor_is_one),
    ("V168 D 的门槛不含签发", check_release_check_excludes_the_issue),
    ("PR1 三份已有产物在时逐行复现 r3 第一段 2684 行", check_real_products_reproduce_the_r3_product_when_present),
)


def run_checks():
    red = 0
    for label, check in CHECKS:
        try:
            check()
        except Exception as error:  # 任何异常都算这一项红
            red += 1
            print(f"  ✗ 红：{label}（{type(error).__name__}：{str(error)[:200]}）")
            print("     → 怎么办：按这一项的名字找 e185_r4_baseline_rejudge.py 里对应的判法改，改完重跑这份自证")
        else:
            print(f"  ✓ 绿：{label}")
    print(f"自证 {len(CHECKS)} 项，红 {red} 项")
    return 1 if red else 0


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(run_checks())
