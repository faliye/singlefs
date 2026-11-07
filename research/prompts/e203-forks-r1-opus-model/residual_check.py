"""E203 岔路第一轮攻方腿的小模型：只读计数、计时产物与 Q6 那几行，不编译、不动 crates/。

用法：python3 -I residual_check.py --results <仓根>/research/results --q6 <模型目录>/inputs/q6_lines.txt
打出五组行：
  b8_without_mapped_units_queries  Q6 的 B8 合计减去 `mapped_units_whole_table` 的查询项（Σ query_levels）之后，
                                    记录轴两端的增长阶（同 Q6：α = ln(高/低) ÷ ln(10⁴/10)，≥ 0.5 随、≤ 0.25 不随）
  b1_scan_total_on_the_u_axis      u 轴逐点的 B1 准入之前扫类合计（三段、类型「扫」各行，同 Q2）与其中整份拷几份
  release_find_calls               `allocator_release_linear_find` 计数构建记的调用数与走查次数（调用处表长之和 ÷ 记录数）
  caller_copy_unit_area_part       调用方那一份整份拷的步数减去只由 U 定的那一部分（2 × (3U + 5⌈U/64⌉)）
  x2_timing                        X2 上调用方那一份的独占计时、两道之间那一段与其中整份拷的独占计时
  s2_residual_on_the_u_axis        两道之间那一段的残差（同 Q10）在 U 轴两端五轮各自的增长阶
"""
import argparse
import math
import os

NOT_SCAN_ROWS = {
    "inode_leaf_container_copy", "inode_leaf_writer_copy", "files_other_than_the_first_lower_node_copy",
    "tombstone_container_copy", "retirement_container_copy", "lineage_container_copy",
    "units_of_a_publish_to_land", "unlisted_index_or_plan_work",
}
BEFORE_ADMISSION = ("caller", "before_landing", "between")
AXIS_HIGH_END = {"D": "data_units_10000-2026-11-06", "F": "files_10000-2026-11-06"}
LOW_END = "base-2026-11-06-r2"
U_AXIS = [("1", "unlinks_1-2026-11-06-r2"), ("10", LOW_END), ("100", "unlinks_100-2026-11-06-r2"),
          ("1000", "unlinks_1000-2026-11-06-r2"), ("10000", "unlinks_10000-2026-11-06-r2")]
X2 = "user_scale_unit_area-2026-11-07"


def fields_of(line):
    return dict(token.split("=", 1) for token in line.split() if "=" in token)


def lines_of(results, build, point, name):
    path = os.path.join(results, f"e203-admission-scan-cost-{build}-{point}.out")
    with open(path, encoding="utf-8") as product:
        for line in product:
            if line.startswith("E203RESULT ") and f" name={name} " in line:
                yield fields_of(line)


def rows_of(results, build, point, state, kind, round_number="1"):
    return [row for row in lines_of(results, build, point, "row")
            if row["state"] == state and row["kind"] == kind and row["round"] == round_number]


def query_term(results, point, state, kind):
    return sum(int(row["query_levels"]) for row in rows_of(results, "counts", point, state, kind)
               if row["row"] == "mapped_units_whole_table" and row["segment"] in BEFORE_ADMISSION)


def band(alpha):
    return "GrowsWithTheAxis" if alpha >= 0.5 else ("DoesNotGrow" if alpha <= 0.25 else "Between")


def b8_without_queries(results, q6_path):
    with open(q6_path, encoding="utf-8") as q6:
        cells = [fields_of(line) for line in q6 if line.startswith("name=q6 ")]
    for cell in cells:
        if cell["axis"] not in AXIS_HIGH_END:
            continue
        state, kind = cell["state"], cell["kind"]
        low_term = query_term(results, LOW_END, state, kind)
        high_term = query_term(results, AXIS_HIGH_END[cell["axis"]], state, kind)
        low, high = int(cell["low_end"]) - low_term, int(cell["high_end"]) - high_term
        alpha = math.log(high / low) / math.log(1000)
        print(f"name=b8_without_mapped_units_queries axis={cell['axis']} state={state} kind={kind} "
              f"low_end={cell['low_end']} high_end={cell['high_end']} query_term_low={low_term} "
              f"query_term_high={high_term} residual_low={low} residual_high={high} alpha={alpha:.3f} band={band(alpha)}")


def scan_total_on_the_u_axis(results):
    for state in ("same_mount", "remounted"):
        previous, decreases = None, 0
        for unlinks, point in U_AXIS:
            rows = [row for row in rows_of(results, "counts", point, state, "overwrite")
                    if row["segment"] in BEFORE_ADMISSION and row["row"] not in NOT_SCAN_ROWS]
            total = sum(int(row["steps"]) for row in rows)
            copies = sum(int(row["calls"]) for row in rows if row["row"].startswith("whole_allocator_copy"))
            print(f"name=b1_scan_total_on_the_u_axis state={state} kind=overwrite unlinks={unlinks} "
                  f"scan_total={total} whole_allocator_copies={copies}")
            if previous is not None and total < previous:
                decreases += 1
            previous = total
        print(f"name=monotone_on_the_u_axis state={state} kind=overwrite steps_where_the_total_decreased={decreases}")


def release_find_calls(results):
    for point in (LOW_END, AXIS_HIGH_END["D"], AXIS_HIGH_END["F"], X2):
        for state in ("same_mount", "remounted"):
            records = [int(s["allocation_records"]) for s in lines_of(results, "counts", point, "state")
                       if s["state"] == state and s["kind"] == "overwrite" and s["round"] == "1"][0]
            for row in rows_of(results, "counts", point, state, "overwrite"):
                if row["row"] == "allocator_release_linear_find" and row["segment"] == "between":
                    scale, calls = int(row["scale_to_the_hit"]), int(row["calls"])
                    print(f"name=release_find_calls point={point} state={state} kind=overwrite round=1 "
                          f"counted_calls={calls} records={records} scale_to_the_hit={scale} "
                          f"finds={scale / records:.3f} finds_over_counted_calls={scale / records / calls:.3f}")


def caller_copy_unit_area_part(results):
    points = [LOW_END, AXIS_HIGH_END["D"], AXIS_HIGH_END["F"], "unlinks_10000-2026-11-06-r2", "diagonal-2026-11-06",
              "unit_area_2_pow_20-2026-11-06-r2", "unit_area_2_pow_24-2026-11-06-r2", X2]
    for point in points:
        unit_area = int(next(lines_of(results, "counts", point, "point"))["unit_area_slots"])
        part = 2 * (3 * unit_area + 5 * -(-unit_area // 64))
        for state in ("same_mount", "remounted"):
            for row in rows_of(results, "counts", point, state, "overwrite"):
                if row["row"] == "whole_allocator_copy_before_the_judgement":
                    steps = int(row["steps"])
                    print(f"name=caller_copy_unit_area_part point={point} state={state} unit_area_slots={unit_area} "
                          f"steps={steps} from_the_unit_area={part} other_tables={steps - part}")


def x2_timing(results):
    for state in ("same_mount", "remounted"):
        for round_number in ("1", "2", "3", "4", "5"):
            rows = rows_of(results, "timing", X2, state, "overwrite", round_number)
            caller = sum(int(r["exclusive_nanos"]) for r in rows if r["row"] == "whole_allocator_copy_before_the_judgement")
            settling = sum(int(r["exclusive_nanos"]) for r in rows if r["row"] == "whole_allocator_copy_in_a_settling_round")
            between = [int(s["nanos"]) for s in lines_of(results, "timing", X2, "segment_nanos")
                       if s["state"] == state and s["kind"] == "overwrite" and s["round"] == round_number
                       and s["segment"] == "between"][0]
            print(f"name=x2_timing state={state} kind=overwrite round={round_number} caller_copy_nanos={caller} "
                  f"between_nanos={between} settling_copy_nanos={settling} settling_share={settling / between:.3f}")


def s2_residual_on_the_u_axis(results):
    """两道之间那一段的残差（段挂钟减这一段各行独占计时之和，同登记 Q10）在 U 轴两端 2¹⁴ → 2²⁴ 上五轮各自的 α。"""
    ends = ("unit_area_2_pow_14-2026-11-06-r2", "unit_area_2_pow_24-2026-11-06-r2")
    for kind in ("overwrite", "unlink_intent", "unlink_final"):
        for state in ("same_mount", "remounted"):
            alphas = []
            for round_number in ("1", "2", "3", "4", "5"):
                residuals = []
                for point in ends:
                    wall = [int(s["nanos"]) for s in lines_of(results, "timing", point, "segment_nanos")
                            if s["state"] == state and s["kind"] == kind and s["round"] == round_number
                            and s["segment"] == "between"][0]
                    exclusive = sum(int(r["exclusive_nanos"]) for r in rows_of(results, "timing", point, state, kind, round_number)
                                    if r["segment"] == "between")
                    residuals.append(wall - exclusive)
                alphas.append(math.log(residuals[1] / residuals[0]) / math.log(2 ** 10))
            print(f"name=s2_residual_on_the_u_axis state={state} kind={kind} "
                  f"alphas={','.join(f'{a:.3f}' for a in alphas)} all_five_at_least_half={all(a >= 0.5 for a in alphas)}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--results", required=True)
    parser.add_argument("--q6", required=True)
    arguments = parser.parse_args()
    b8_without_queries(arguments.results, arguments.q6)
    scan_total_on_the_u_axis(arguments.results)
    release_find_calls(arguments.results)
    caller_copy_unit_area_part(arguments.results)
    x2_timing(arguments.results)
    s2_residual_on_the_u_axis(arguments.results)


if __name__ == "__main__":
    main()
