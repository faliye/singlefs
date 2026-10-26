#!/usr/bin/env python3
# admission: always 每一次调都跑，判的是同目录 e185_r1_compare.py 此刻的样子，上一次的结论不替这一次作保
# run-condition: none 只在合成产物（临时目录）和上一趟两份产物（在工作区时）上读文本，除了 python3 之外没有环境要求，不起 cargo
"""e185_r1_compare.py 的单测（E185 第 1 次重跑登记第九节「比法脚本的变异」九行各有一项点名要红的自证）。

用法：python3 scripts/experiment-apparatus/e185_r1_compare_test.py     每项一行 ✓ 绿 / ✗ 红，末行「自证 N 项，红 M 项」，有红退 1。
"""
import os
import sys
import tempfile

import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
from e185_r1_compare import *  # noqa: E402,F401,F403
from e185_r1_compare import Unreadable  # noqa: E402

# ── 自证用的合成产物（字段的写法照第一段、第二段装置的产物行） ────────────────────

def synthetic_intent(fraction, pool, batch, round_number, valid="true", arm="memory"):
    return {"name": "intent", "arm": arm, "fraction": fraction, "pool_data_units": pool, "batch_units": batch,
            "round": str(round_number), "attempt": "1", "injection": "none", "valid": valid, "batches": "3"}


def synthetic_call(fraction, pool, batch, round_number, index, kind, cpu, arm="memory", release="524288", unreadable="false"):
    copy = "0" if kind == "issue" else str(2 * min(int(batch), MOVABLE_UNITS[fraction]) * UNIT_BYTES)
    return {"name": "call", "arm": arm, "fraction": fraction, "pool_data_units": pool, "batch_units": batch,
            "round": str(round_number), "attempt": "1", "injection": "none", "call_index": str(index), "kind": kind,
            "cpu_nanoseconds": repr(float(cpu)), "scan_read_bytes_in_the_region": str(ISSUE_SCAN_BYTES[fraction]),
            "release_check_read_bytes": "0" if kind == "issue" else release, "relocation_copy_bytes": copy,
            "write_bytes": "1000", "metadata_write_bytes": "500", "cpu_unreadable": unreadable}


def synthetic_round(fraction, pool, batch, round_number, cpu, valid="true", arm="memory", unreadable="false", release="524288"):
    """一轮：一个 intent、签发那一次（call_index 0，搬 0 个单元）、一个批、最后一批。"""
    return [synthetic_intent(fraction, pool, batch, round_number, valid, arm),
            synthetic_call(fraction, pool, batch, round_number, 0, "issue", cpu / 100, arm, release, unreadable),
            synthetic_call(fraction, pool, batch, round_number, 1, "batch", cpu, arm, release, unreadable),
            synthetic_call(fraction, pool, batch, round_number, 2, "final_batch", cpu * 1.1, arm, release, unreadable)]


def synthetic_product(segment, cpu_by_pool, fractions=("0.75",), batches=("4", "8"), q1_judgement="cpu", release="524288"):
    """cpu_by_pool = {"1024": [第 1 轮, 第 2 轮, …], "8192": […]}；每个（fraction, batch_units）用同样的数。"""
    rows = [{"name": "configuration", "segment": segment, "image_directory_filesystem": "ext4", "image_device": "64512"}]
    for fraction in fractions:
        for pool in cpu_by_pool:
            rows.append({"name": "geometry", "fraction": fraction, "pool_data_units": pool, "choice": "forced_fallback", "formed": "true"})
            rows.append({"name": "geometry_attempt", "fraction": fraction, "pool_data_units": pool, "mapped_units": str(int(pool) + 34)})
        for batch in batches:
            for pool, values in cpu_by_pool.items():
                for round_number, cpu in enumerate(values, 1):
                    rows.extend(synthetic_round(fraction, pool, batch, round_number, cpu, release=release))
                    if segment == "stage_split":
                        rows.append({"name": "stage", "fraction": fraction, "pool_data_units": pool, "batch_units": batch,
                                     "round": str(round_number), "call_index": "1", "kind": "batch", "call_cpu_nanoseconds": repr(float(cpu)),
                                     "scan_cpu_nanoseconds": "100.000000", "crc_share": "0.001000"})
            if segment == "wall_clock":
                rows.append({"name": "q1", "fraction": fraction, "pool_data_units": POOL_SIZE_SMALL,
                             "batch_units": batch, "rho_median": "2.9", "judgement": q1_judgement})
    return rows


def render(rows):
    lines = ["E7RESULT " + " ".join(f"{key}={value}" for key, value in row.items() if not key.startswith("_")) for row in rows]
    lines.append(f"E7RESULT name=done emitted={len(lines) + 1}")
    return "\n".join(lines) + "\n"


def write_product(directory, label, segment, cpu_by_pool, **kwargs):
    path = os.path.join(directory, label)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(render(synthetic_product(segment, cpu_by_pool, **kwargs)))
    return path


def values_of(rows):
    return wall_clock_series(rows)[0]


# ── 自证：每一项的标签就是登记第九节变异表点名要红的那一格 ───────────────────────

def build_rows(small, large, fraction="0.75", batch="8", arm="memory"):
    rows = []
    for pool, series in ((POOL_SIZE_SMALL, small), (POOL_SIZE_LARGE, large)):
        for round_number, cpu in enumerate(series, 1):
            rows.extend(synthetic_round(fraction, pool, batch, round_number, cpu, arm=arm))
    return rows


def check_exponent_anchor():
    assert exponent(1, 8) == 1.0
    assert exponent(1, 64) == 2.0
    assert exponent(2, 16) == 1.0
    cell = exponent_cells(values_of(build_rows([1.0] * 5, [64.0] * 5)))[("0.75", "8")]
    assert cell["alpha"] == 2.0 and cell["judgement"] == "near_square_between_the_two_points", cell


def check_the_batch_read_moved_k_units():
    rows = synthetic_product("wall_clock", {POOL_SIZE_SMALL: [100.0] * 5, POOL_SIZE_LARGE: [800.0] * 5}, batches=("4", "8", "16"))
    cells = anchor_cells(rows)
    for batch, expected in (("4", 262144), ("8", 524288), ("16", 1048576)):
        cell = cells[("0.75", POOL_SIZE_SMALL, batch)]
        assert cell["status"] == "ok" and cell["copy_seen"] == [str(expected)] and cell["mismatches"] == 0, cell
    assert values_of(rows)[("0.75", "8")][POOL_SIZE_SMALL][1] == 100.0, "取到的是签发那一次，不是搬了 K 个单元的批"
    for row in rows:
        if row["name"] == "call" and row["kind"] == "batch" and row["batch_units"] == "8":
            row["relocation_copy_bytes"] = "0"
    assert anchor_cells(rows)[("0.75", POOL_SIZE_SMALL, "8")]["status"] == "mismatch"


def check_invalid_rounds_dropped():
    rows = []
    for round_number in range(1, 6):
        valid, cpu = ("true", 100.0) if round_number <= 2 else ("false", 1000.0)
        rows.extend(synthetic_round("0.75", POOL_SIZE_SMALL, "8", round_number, cpu, valid=valid))
    series = values_of(rows)[("0.75", "8")][POOL_SIZE_SMALL]
    assert series == {1: 100.0, 2: 100.0}, series
    assert median_of(list(series.values())) == 100.0


def check_threshold_edges_are_inclusive():
    assert exponent(1, 13.454342644059432) == 1.25
    assert judge_exponent(exponent(1, 13.454342644059432)) == "near_linear_between_the_two_points"
    assert judge_exponent(exponent(1, 38.05462768008707)) == "near_square_between_the_two_points"
    assert judge_exponent(1.25) == "near_linear_between_the_two_points" and judge_exponent(1.75) == "near_square_between_the_two_points"
    assert judge_exponent(1.2500001) == "between_the_two" and judge_exponent(1.7499999) == "between_the_two"


def check_memory_arm_only():
    rows = []
    for round_number in range(1, 6):
        rows.extend(synthetic_round("0.75", POOL_SIZE_SMALL, "8", round_number, 200.0, arm="real_disk"))
        rows.extend(synthetic_round("0.75", POOL_SIZE_SMALL, "8", round_number, 100.0))
    series = values_of(rows)[("0.75", "8")][POOL_SIZE_SMALL]
    assert len(series) == 5 and set(series.values()) == {100.0}, series


def check_same_work_compares_the_release_check_bytes():
    pools = {POOL_SIZE_SMALL: [100.0] * 5, POOL_SIZE_LARGE: [800.0] * 5}
    base = synthetic_product("wall_clock", pools, batches=("8",))
    other = synthetic_product("wall_clock", pools, batches=("8",), release="262144")
    assert {cell["judgement"] for cell in same_work_cells(base, base).values()} == {"same_work"}
    cells = same_work_cells(base, other)
    assert cells and all(cell["judgement"] == "different_work" for cell in cells.values()), cells
    assert any("release_check_read_bytes" in text for cell in cells.values() for text in cell["differences"])


def check_per_round_pairing_is_by_round():
    cell = exponent_cell({1: 1.0, 2: 2.0, 3: 3.0, 4: 4.0, 5: 5.0}, {1: 8.0, 2: 16.0, 3: 24.0, 4: 32.0, 5: 40.0})
    assert cell["per_round"] == {number: 1.0 for number in range(1, 6)}, cell["per_round"]
    assert cell["stable"] and cell["alpha"] == 1.0 and cell["judgement"] == "near_linear_between_the_two_points"


def check_median_is_not_the_mean():
    cell = exponent_cell({1: 1.0, 2: 1.0, 3: 1.0, 4: 1.0, 5: 100.0}, {1: 8.0, 2: 8.0, 3: 8.0, 4: 8.0, 5: 8.0})
    assert cell["small_median"] == 1.0 and cell["large_median"] == 8.0 and cell["alpha"] == 1.0, cell
    assert median_of([1, 1, 1, 1, 100]) == 1


def check_moving_the_threshold_flips_the_judgement():
    assert judge_exponent(1.3) == "between_the_two"
    assert judge_exponent(1.3, near_linear_limit=1.4) == "near_linear_between_the_two_points"
    assert judge_exponent(1.6) == "between_the_two"
    assert judge_exponent(1.6, near_square_limit=1.5) == "near_square_between_the_two_points"


def check_unreadable_is_not_zero():
    rows = build_rows([100.0] * 5, [800.0] * 5)
    for row in rows:
        if row["name"] == "call" and row["pool_data_units"] == POOL_SIZE_SMALL and row["round"] == "3":
            row["cpu_unreadable"] = "true"
    values, unreadable = wall_clock_series(rows)
    assert 3 not in values[("0.75", "8")][POOL_SIZE_SMALL] and unreadable == {("0.75", "8", POOL_SIZE_SMALL): 1}, (values, unreadable)
    assert exponent(0, 8) is None and judge_exponent(None) == "not_readable"
    assert exponent_cell({}, {1: 8.0})["judgement"] == "not_readable"


def run_in(directory, before, after=None, expected_override=None, inputs_list=None):
    """before、after 是 {"wall_clock": (cpu_by_pool, kwargs), "stage_split": …}；返回 (退出码, 输出文本)。"""
    paths = {}
    for side, spec in (("before", before), ("after", after or {})):
        for segment, (cpu_by_pool, kwargs) in spec.items():
            paths[f"{side}_{segment}"] = write_product(directory, f"{side}-{segment}.out", segment, cpu_by_pool, **kwargs)
    expected = [file_sha256(paths["before_wall_clock"]), file_sha256(paths["before_stage_split"])]
    if expected_override:
        expected[0] = expected_override
    options = dict(paths, inputs_list=inputs_list)
    lines = []
    code = run_comparison(options, lines, expected[0], expected[1])
    return code, "\n".join(lines)


LINEAR = {POOL_SIZE_SMALL: [1000.0] * 5, POOL_SIZE_LARGE: [8000.0] * 5}
SQUARE = {POOL_SIZE_SMALL: [1000.0] * 5, POOL_SIZE_LARGE: [64000.0] * 5}


def check_missing_side_is_not_run():
    with tempfile.TemporaryDirectory(prefix="e185-r1-compare-test-") as directory:
        code, text = run_in(directory, {"wall_clock": (SQUARE, {}), "stage_split": (SQUARE, {})})
    assert code == 0, text
    assert "name=same_work cell=C0 segment=wall_clock judgement=not_run" in text, text
    assert "after_wall_clock=not_run after_stage_split=not_run" in text and "g1_after_not_near_linear=not_run" in text
    assert " side=after" not in text and text.splitlines()[-1].startswith("E7RESULT name=done emitted=")
    assert "alpha=2.000000 judgement=near_square_between_the_two_points" in text


def check_changed_baseline_stops():
    with tempfile.TemporaryDirectory(prefix="e185-r1-compare-test-") as directory:
        code, text = run_in(directory, {"wall_clock": (SQUARE, {}), "stage_split": (SQUARE, {})}, expected_override="0" * 64)
    assert code == 3 and "name=stopped" in text and "name=cpu_exponent" not in text, text


def check_before_and_after_end_to_end():
    with tempfile.TemporaryDirectory(prefix="e185-r1-compare-test-") as directory:
        code, text = run_in(directory, {"wall_clock": (SQUARE, {}), "stage_split": (SQUARE, {})},
                            {"wall_clock": (LINEAR, {"q1_judgement": "disk"}), "stage_split": (LINEAR, {})})
    assert code == 0, text
    assert "g1_after_not_near_linear=0 g2_before_not_near_square=0 g3_two_sources_disagree_before=0 g3_two_sources_disagree_after=0 g4_different_work=0 g5_flipped=2" in text, text
    assert "name=cpu_ratio cell=C2 source=wall_clock_call_cpu fraction=0.75 pool_data_units=8192 batch_units=8 same_work=same_work after_over_before=0.125000" in text
    assert "name=cpu_ratio cell=C2 source=stage_split_remainder fraction=0.75 pool_data_units=1024 batch_units=8 same_work=same_work after_over_before=1.000000" in text
    assert "flipped=true" in text and "output_void=false" in text


def check_inputs_list_and_after_product():
    assert inputs_list_problems(f"sha {DEVICE_SHA256}\ncargo tree singlefs-core feature default") == []
    assert inputs_list_problems("no device here") == ["device_sha256_not_in_the_list"]
    assert inputs_list_problems(f"{DEVICE_SHA256} {COUNTER_FEATURE}") == ["counter_feature_in_the_cargo_tree"]
    with tempfile.TemporaryDirectory(prefix="e185-r1-compare-test-") as directory:
        path = write_product(directory, "same.out", "wall_clock", LINEAR)
        stage = write_product(directory, "same-stage.out", "stage_split", LINEAR)
        options = {"before_wall_clock": path, "before_stage_split": stage, "after_wall_clock": path}
        try:
            run_comparison(options, [], file_sha256(path), file_sha256(stage))
        except Unreadable:
            return
    raise AssertionError("「后」的产物就是上一趟的那一份，应当拒")


REAL_BASELINES = ("research/results/e185-gc-batch-wall-clock-split-wall-clock-2026-10-20.out",
                  "research/results/e185-gc-batch-wall-clock-split-stage-split-2026-10-20.out")


def check_real_baseline_products_when_present():
    root = os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..")
    paths = [os.path.join(root, each) for each in REAL_BASELINES]
    if not all(os.path.exists(each) for each in paths):
        print("    （上一趟两份产物不在工作区，这一项跳过）")
        return
    lines = []
    code = run_comparison({"before_wall_clock": paths[0], "before_stage_split": paths[1]}, lines)
    text = "\n".join(lines)
    assert code == 0, text
    assert sum(1 for each in lines if "name=cpu_exponent" in each and "side=before" in each and "status=not_run" not in each) == 18
    assert "anchor_mismatches=0" in text and "baseline_sha_mismatches=0" in text


CHECKS = (
    ("指数函数锚点", check_exponent_anchor),
    ("第一批是搬了 K 个单元的那一批", check_the_batch_read_moved_k_units),
    ("剔除作废轮", check_invalid_rounds_dropped),
    ("门槛两条边含在哪一侧", check_threshold_edges_are_inclusive),
    ("只取内存臂", check_memory_arm_only),
    ("同一份活要比释放前读盘核那一栏", check_same_work_compares_the_release_check_bytes),
    ("每轮指数按同轮配对", check_per_round_pairing_is_by_round),
    ("中位不是平均", check_median_is_not_the_mean),
    ("判别力自证：门槛挪到两个量到的指数之间", check_moving_the_threshold_flips_the_judgement),
    ("读不到不当 0", check_unreadable_is_not_zero),
    ("缺的一侧报本次未跑", check_missing_side_is_not_run),
    ("前的产物 sha256 不对就停", check_changed_baseline_stops),
    ("前后端到端", check_before_and_after_end_to_end),
    ("输入清单与后的产物不能是上一趟的", check_inputs_list_and_after_product),
    ("上一趟真产物在时锚点与行数", check_real_baseline_products_when_present),
)


def run_checks():
    red = 0
    for label, check in CHECKS:
        try:
            check()
        except Exception as error:  # 任何异常都算这一项红
            red += 1
            print(f"  ✗ 红：{label}（{type(error).__name__}：{str(error)[:200]}）")
            print("     → 怎么办：按这一项的名字找 e185_r1_compare.py 里对应的判法改，改完重跑这份自证")
        else:
            print(f"  ✓ 绿：{label}")
    print(f"自证 {len(CHECKS)} 项，红 {red} 项")
    return 1 if red else 0


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(run_checks())
