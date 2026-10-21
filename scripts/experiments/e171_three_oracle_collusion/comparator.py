"""E171（三oracle合谋测试） 比对器：读装置、逐行驱动、冻结的 O3 执行器留下的导出目录与判定，写结果行（登记第六节、第七节、第十一节）。

跑法（在本目录，经 run-with-memory-cap.sh 与 capped.sh；不起子进程跑编译出来的代码，只在 A6 起挑法脚本）：
  python3 comparator.py --output <新文件> --mutations-table <crates/mutations.tsv>
      --baseline-dir <基线导出> --baseline-device-output <基线装置的标准输出> [--baseline-repeat-dir <第二次基线导出>] [--e170-dump <E170 导出>]
      [--o3-baseline <冻结执行器在基线导出上的产物>] [--driver-output <逐行驱动的标准输出> …]
      [--control <O1-对拍|O1-崩溃判官|O2|O3>:<baseline|empty_mutation>:<导出目录；O3 是 o3-control.out> …] [--disabled <O1|O2>:<关一条的导出目录> …]
      [--rows-root <逐行驱动的输出根>] [--q3-core <O1|O2|no_executor>:<不关的那一趟的输出根>:<关了的那一趟的输出根> …]
      [--known-bad-dir <装置 known-bad-images 的输出> --e170-expected-dir <E170 bad-states 的导出> [--image-judgements <o3_bad_image_judge.py judge 的产物>]]
      [--expansion-rows <F4 扩表>] [--segment <config 行里的段名，默认 s4>]
  python3 comparator.py --print-q3-core-rows --rows-root <逐行驱动的输出根>     打印 Q3-核要跑的三行（登记第六节 Q3-核），不写产物
F4 扩表（登记 5.2「扩表」、F4）：`--rows-root` 给两份（117 行那份与扩表那份），两份里同一个行号出现两次就拒；`--expansion-rows` 给扩表，
多出两行结果：`name=f4_expansion`（扩表与挑法脚本在这张 `--mutations-table` 上现算的是不是同一张）与 `name=q1_denominator_by_table`（分母按表分开报）。
Q1 的分母、甲乙丙与 Q3 都按并起来的全部行算（登记 F4「扩完仍不够记『不够判』」）。
补位（登记第一节读法 5）：`--substitution-rows-root` 给补位那一趟的输出根，里面的行换掉 `--rows-root` 里同一个序号、编不过的那一行（别的情形拒），
多出 `name=substitution`（逐行：原名、换上的行、试过的每一个、换上之后的分类）与 `name=substitution_summary`；分母与甲乙丙、Q3 按换过之后的行算。
O3 是冻结的那一版执行器（E170 第 1 次重跑冻结，登记第一节读法 8）：开头与收尾各核一次冻结行（Z2）。没给 O3 的产物时 O3 那几列写 `not_run`。
"""

from __future__ import annotations

import argparse
import collections
import os
import shutil
import subprocess
import sys
import tempfile
import time

import expansion_table
import frozen_o3
from bad_images_report import SOURCE_BASE, attach_o3, bad_image_lines, read_e170_rows, read_image_judgements, read_known_bad_rows
from baseline_checks import (baseline_red_counts, compare_two_baselines, compare_with_e170_dump,
                             differences_after_closing_one_oracle, differences_when_nothing_is_closed, o3_state_lines_without_timing,
                             recompute_crash_judge_applicable, recompute_first_batch)
from bad_implementation_table import (OWN_INSTRUMENT_ROW_PREFIX, read_mutation_rows, read_selected_rows,
                                      without_the_rows_of_the_device_itself)
from collusion_matrix import (CATEGORY_COUNTED, FIRST_BATCH, errored_calls_left_unjudged, not_applicable_states_with_another_source,
                              o3_judged_state_count, oracles_that_alone_hold_the_row, red_columns, strict_reading_is_unreachable)
from collusion_report import (O3_STATUS_EXECUTOR_FAILED, O3_STATUS_RAN, Q1_READINGS, RowAnalysis, analyse_row, counted_rows, q1_lines,
                              quiet_rows_in_batch, row_lines, timing_lines, untouched_but_red_rows)
from result_lines import ResultLines, field
from row_driver import REBUILD_CHECK_FIELD, STATUS_OK
from run_directory import (STREAM_NAMES, device_stream_summaries, device_verdict_line, load_run_directory, parse_result_line,
                           read_rows)

HERE = os.path.dirname(os.path.abspath(__file__))
EXPECTED_ROWS_PER_FILE = {
    "core/admission.rs": 5, "core/allocation_record_tree.rs": 2, "core/allocator.rs": 8, "core/block_device.rs": 2,
    "core/code_two_tree.rs": 3, "core/extent_tree.rs": 2, "core/inode_tree.rs": 1, "core/instance_table.rs": 2,
    "core/journal.rs": 2, "core/make_filesystem.rs": 4, "core/mount.rs": 26, "core/mounted_read.rs": 3,
    "core/mounted_session.rs": 2, "core/pointer.rs": 1, "core/records.rs": 2, "core/recovery.rs": 17,
    "core/root_record.rs": 1, "core/root_ring.rs": 1, "core/system_configuration.rs": 3, "core/transaction.rs": 20,
    "core/unit.rs": 2, "core/write_accounting.rs": 1, "core/write_request_split.rs": 1, "replay-prefix/lib.rs": 6}
VERDICT_KEYS = ("a3_columns_with_red_on_the_baseline", "z1_files_differing", "s2_files_differing", "s5_numbers_that_differ",
                "positive_controls_run", "positive_controls_not_red", "disabled_runs_with_differences_outside_the_closed_column",
                "q3_core_runs", "q3_core_runs_with_differences", "rows_untouched_but_red", "negative_bases_with_o2_red",
                "negative_bases_with_o3_red", "known_bad_rows_with_o2_quiet", "o3_freeze_differences", "o3_digest_mismatch_states", "o3_dump_problem_lines",
                "o3_image_digest_mismatches", "rows_with_o3_freeze_lines_not_the_reference", "rows_with_the_o3_executor_failed",
                "a2_p_rows_not_expected_not_in", "f3_p_rows_judged_in_by_o3", "q2_geometry_unstable_kinds",
                "q1_alpha_all_quiet_rows", "q1_beta_all_quiet_rows", "q1_gamma_all_quiet_rows", "q1_o3_strict_reading_unreachable",
                "q2_all_quiet_alpha", "q2_all_quiet_beta", "q3_alpha_rows_held_by_a_single_oracle", "q3_beta_rows_held_by_a_single_oracle",
                "rows_with_an_errored_call_the_comparison_did_not_judge", "rows_with_o1_not_applicable_from_another_source",
                "rows_compiled_without_the_rebuild_check")
CONTROL_ORACLES = ("O1-对拍", "O1-崩溃判官", "O2", "O3")
Q3_CORE_CLOSINGS = ("O1", "O2", "no_executor")
Q3_CORE_ROWS = 3
FROZEN_EXECUTOR_OUTPUT = "o3.out"
O3_CONTROL_OUTPUT = "o3-control.out"
CONTROL_KINDS = ("baseline", "empty_mutation")


def registered_table_lines(mutations_table: str, selected_path: str, lines: ResultLines) -> None:
    """A4–A7：登记 5.9 的表按名字在今天的 `crates/mutations.tsv` 里逐行核，再用挑法脚本现算步长 8 与步长 4 的行数（命令与登记第十三节同形）。"""
    selected = read_selected_rows(selected_path)
    table = read_mutation_rows(mutations_table)
    by_name: dict[str, list] = {}
    for row in table:
        by_name.setdefault(row.name, []).append(row)
    found_once = sum(len(by_name.get(row.name, [])) == 1 for row in selected)
    same_line = sum(len(by_name.get(row.name, [])) == 1 and by_name[row.name][0].line_digest_prefix() == row.line_digest_prefix
                    for row in selected)
    per_file: dict[str, int] = {}
    for row in selected:
        key = "replay-prefix/lib.rs" if row.file.startswith("crates/singlefs-replay-prefix") else row.file.removeprefix("crates/singlefs-").replace("/src/", "/")
        per_file[key] = per_file.get(key, 0) + 1
    lines.result(f"name=a4 table_rows={len(selected)} expected=117 per_file_matches_the_registration={str(per_file == EXPECTED_ROWS_PER_FILE).lower()} "
                 f"found_once_in_today_table={found_once} line_unchanged_since_the_registration={same_line}")
    lines.result(f"name=a5 first_slice_rows={sum(row.slice_number == 1 for row in selected)} expected=24 "
                 f"distinct_files={len({row.file for row in selected if row.slice_number == 1})}")
    script = os.path.join(HERE, "select_bad_implementations.py")
    with open(mutations_table, encoding="utf-8") as handle:
        table_text_without_the_device_rows, device_rows = without_the_rows_of_the_device_itself(handle.read())
    scratch_directory = tempfile.mkdtemp(prefix="e171-table-without-the-device-rows-")
    table_without_the_device_rows = os.path.join(scratch_directory, "mutations.tsv")
    with open(table_without_the_device_rows, "w", encoding="utf-8") as handle:
        handle.write(table_text_without_the_device_rows)

    def selected_names(step: int, extra: list[str]) -> list[str]:
        output = subprocess.run([sys.executable, script, table_without_the_device_rows, str(step)] + extra, check=True, capture_output=True, text=True).stdout
        return [line.split("\t")[3] for line in output.splitlines()]
    step_eight, step_four = selected_names(8, []), selected_names(4, [])
    shutil.rmtree(scratch_directory)
    lines.result(f"name=a6 step_8_rows={len(step_eight)} step_4_rows={len(step_four)} expected_step_4=220 "
                 f"overlap={len(set(step_eight) & set(step_four))} expected_overlap=56 step_4_only={len(set(step_four) - set(step_eight))} expected_step_4_only=164 "
                 f"step_8_names_equal_the_registered_table={str(sorted(step_eight) == sorted(row.name for row in selected)).lower()}")
    implementation_table = [row for row in table if not row.name.startswith(OWN_INSTRUMENT_ROW_PREFIX)]
    implementation_rows = sum(row.file.startswith("crates/singlefs-core/src/") or row.file == "crates/singlefs-replay-prefix/src/lib.rs" for row in implementation_table)
    lines.result(f"name=a7 implementation_rows={implementation_rows} expected=844 files={len({row.file for row in implementation_table if row.file.startswith('crates/singlefs-core/src/') or row.file == 'crates/singlefs-replay-prefix/src/lib.rs'})} expected_files=24")
    lines.result(f"name=a7_device_rows device_rows_left_out_of_a6_and_a7={device_rows}")


def expansion_table_lines(mutations_table: str, selected_path: str, expansion_path: str, lines: ResultLines) -> None:
    """F4 扩表（登记 5.2「扩表」）：入库的扩表与挑法脚本在这张表上现算的逐行相同（序号、刀、文件、组内下标、名字、行文摘要）、行数 164、
    序号接在 117 之后、刀号都是 3、与 117 行一个名字都不重叠、每个名字在这张表里恰好一行。"""
    expansion = read_selected_rows(expansion_path)
    with open(mutations_table, encoding="utf-8") as handle:
        recomputed = expansion_table.expansion_rows(handle.read())
    registered_names = {row.name for row in read_selected_rows(selected_path)}
    names_in_the_table = collections.Counter(row.name for row in read_mutation_rows(mutations_table))
    numbers = [row.number for row in expansion]
    lines.result(f"name=f4_expansion rows={len(expansion)} expected={expansion_table.EXPECTED_EXPANSION_ROWS} "
                 f"first_number={numbers[0] if numbers else 'none'} last_number={numbers[-1] if numbers else 'none'} "
                 f"numbers_are_consecutive={str(numbers == list(range(expansion_table.FIRST_EXPANSION_ROW_NUMBER, expansion_table.FIRST_EXPANSION_ROW_NUMBER + len(numbers)))).lower()} "
                 f"slice_3_rows={sum(row.slice_number == expansion_table.EXPANSION_SLICE_NUMBER for row in expansion)} "
                 f"names_shared_with_the_117_rows={len({row.name for row in expansion} & registered_names)} "
                 f"found_once_in_today_table={sum(names_in_the_table[row.name] == 1 for row in expansion)} "
                 f"equal_to_the_selection_script_on_today_table={str(expansion == recomputed).lower()}")


def q1_denominator_by_table_lines(rows: list[RowAnalysis], expansion_numbers: set[int], lines: ResultLines) -> None:
    """F4 扩表那一次，Q1 的分母按表分开再报一遍：117 行那张（步长 8）与扩表（步长 4 独有）各自跑了几行、计入分母几行、是哪几行。"""
    for label, members in (("step_8_table", [row for row in rows if row.number not in expansion_numbers]),
                           ("step_4_expansion", [row for row in rows if row.number in expansion_numbers])):
        counted = counted_rows(members)
        lines.result(f"name=q1_denominator_by_table table={label} rows_run={len(members)} counted_D={len(counted)} "
                     f"counted_row_numbers={','.join(str(row.number) for row in counted) or 'none'}")


def baseline_lines(baseline_dir: str, baseline_device_output: str, repeat_dir: str | None, e170_dump: str | None, lines: ResultLines,
                   tally: dict[str, int]) -> None:
    counts = baseline_red_counts(baseline_dir)
    lines.result("name=a3_baseline_reds " + " ".join(f"{key}={value}" for key, value in counts.items()))
    tally["a3_columns_with_red_on_the_baseline"] = sum(value > 0 for value in counts.values())
    verdict = device_verdict_line(baseline_device_output)
    lines.result("name=baseline_device_verdict " + " ".join(f"{key}={field(value)}" for key, value in verdict.items() if key != "name"))
    summaries = device_stream_summaries(baseline_device_output)
    for stream in STREAM_NAMES:
        directory = os.path.join(baseline_dir, stream)
        recomputed = recompute_first_batch(directory)
        states = read_rows(os.path.join(directory, "states.tsv"))
        device_first_batch = {int(columns[1]) for columns in states if columns[3] == "1"}
        lines.result(f"name=s5_batch_1_states stream={stream} recomputed={len(recomputed)} device={len(device_first_batch)} "
                     f"equal_counts={str(len(recomputed) == len(device_first_batch)).lower()} equal_ordinals={str(recomputed == device_first_batch).lower()}")
        tally["s5_numbers_that_differ"] += recomputed != device_first_batch
        device_applicable = {}
        for line in open(baseline_device_output, encoding="utf-8"):
            fields = parse_result_line(line.rstrip("\n"))
            if fields and fields.get("name") == "batch" and fields.get("stream") == stream:
                device_applicable[int(fields["batch"])] = int(fields["o1_crash_judge_green"]) + int(fields["o1_crash_judge_red"])
        for batch in (1, 2, 3):
            recomputed_applicable = recompute_crash_judge_applicable(directory, batch)
            lines.result(f"name=s5_o1_applicable stream={stream} batch={batch} recomputed={recomputed_applicable} device={device_applicable.get(batch, 'none')} "
                         f"equal={str(recomputed_applicable == device_applicable.get(batch)).lower()}")
            tally["s5_numbers_that_differ"] += recomputed_applicable != device_applicable.get(batch)
        lines.result(f"name=baseline_stream stream={stream} ending={summaries.get(stream, {}).get('ending', 'unknown')} "
                     f"sampled_states={summaries.get(stream, {}).get('sampled_states', 'unknown')}")
    if repeat_dir is not None:
        results = compare_two_baselines(baseline_dir, repeat_dir)
        for stream, file_name, identical in results:
            lines.result(f"name=z1 stream={stream} file={file_name} identical={str(identical).lower()}")
        lines.result(f"name=z1_summary files_compared={len(results)} files_differing={sum(not identical for _, _, identical in results)}")
        tally["z1_files_differing"] = sum(not identical for _, _, identical in results)
    if e170_dump is not None:
        results = compare_with_e170_dump(baseline_dir, e170_dump)
        for stream, file_name, identical, prefix in results:
            lines.result(f"name=s2 stream={stream} file={file_name} identical_to_the_e170_export={str(identical).lower()} sha256_prefix16={prefix}")
        lines.result(f"name=s2_summary files_compared={len(results)} files_differing={sum(not identical for _, _, identical, _ in results)}")
        tally["s2_files_differing"] = sum(not identical for _, _, identical, _ in results)


def o3_control_red(path: str) -> int:
    """O3 阳性对照（`o3_bad_image_judge.py control` 的产物）：流一全持久态叠 P1 补丁，冻结的执行器严格跑法判「不在」且底态摘要对得上算 1 处红，否则 0。"""
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            fields = parse_result_line(line.rstrip("\n"))
            if fields is not None and fields.get("name") == "o3_control_verdict":
                return int(fields.get("red_strict") == "true")
    return 0


def control_lines(controls: list[str], lines: ResultLines, tally: dict[str, int]) -> None:
    """阳性对照（登记 5.6）：每条 oracle 的对照在基线那一趟与空变异那一趟各跑一遍，各自要红；红要一路带到矩阵（oracle-verdicts.tsv、call-outcomes.tsv）。"""
    for control in controls:
        oracle, kind, directory = control.split(":", 2)
        if oracle not in CONTROL_ORACLES or kind not in CONTROL_KINDS:
            raise ValueError(f"--control 写成 <{'|'.join(CONTROL_ORACLES)}>:<{'|'.join(CONTROL_KINDS)}>:<目录>：{control}")
        if oracle == "O3":
            red = o3_control_red(directory)
            lines.result(f"name=positive_control oracle={oracle} run={kind} red_units={red} red={str(red > 0).lower()}")
            tally["positive_controls_not_red"] += red == 0
            tally["positive_controls_run"] += 1
            continue
        run = load_run_directory(directory)
        if oracle == "O1-对拍":
            red = sum(call.is_a_model_disagreement() for stream in run.streams.values() for call in stream.calls)
        elif oracle == "O1-崩溃判官":
            red = sum(state.crash_judge == "red" and state.batch == 1 for stream in run.streams.values() for state in stream.states)
        else:
            first_stream = run.streams[STREAM_NAMES[0]]
            last_ordinal = max((state.ordinal for state in first_stream.states), default=-1)
            red = sum(state.pool_checker == "red" and state.ordinal == last_ordinal for state in first_stream.states)
        lines.result(f"name=positive_control oracle={oracle} run={kind} red_units={red} red={str(red > 0).lower()}")
        tally["positive_controls_not_red"] += red == 0
        tally["positive_controls_run"] += 1


def disabled_lines(disabled: list[str], baseline_dir: str, lines: ResultLines, tally: dict[str, int]) -> None:
    for item in disabled:
        closed, directory = item.split(":", 1)
        differences = differences_after_closing_one_oracle(baseline_dir, directory, closed)
        tally["disabled_runs_with_differences_outside_the_closed_column"] += bool(differences)
        lines.result(f"name=q3_core closed={closed} run={field(os.path.basename(directory))} differences_outside_the_closed_column={len(differences)} "
                     f"where={field(','.join(differences) or 'none')}")


def read_row_o3(directory: str, driver_fields: dict[str, str]) -> tuple[dict | None, frozen_o3.ExecutorReading | None]:
    """一行的 O3：驱动记 `o3_status=ran` 且 `o3.out` 有完成标记，读冻结执行器的判定；`executor_failed`（执行器进程没跑完）算弃权——O3 判定是空字典（不红），
    行上另标、另列（读法 7：出错算不红、单列）；`skipped_by_request`、`no_exported_stream`、`not_reached` 是 O3 没跑（None）。"""
    status = driver_fields.get("o3_status", "-")
    if status == O3_STATUS_EXECUTOR_FAILED:
        return {}, None
    path = os.path.join(directory, FROZEN_EXECUTOR_OUTPUT)
    if status != O3_STATUS_RAN or not os.path.exists(path):
        return None, None
    reading = frozen_o3.read_executor_output(path)
    return (reading.verdicts, reading) if reading.complete else ({}, reading)


def row_directories(rows_roots: list[str]) -> list[tuple[int, str]]:
    """几个输出根下的 `row-<序号>/` 并成一张（按序号排）；同一个序号在两个根里都有就拒：两次跑的同一行不许静默地留一份。"""
    found: dict[int, str] = {}
    for rows_root in rows_roots:
        for entry in os.listdir(rows_root):
            if not entry.startswith("row-"):
                continue
            number = int(entry.split("-")[1])
            if number in found:
                raise ValueError(f"第 {number} 行在两个输出根里都有：{found[number]} 与 {os.path.join(rows_root, entry)}")
            found[number] = os.path.join(rows_root, entry)
    return sorted(found.items())


def rows_with_substitutes(merged: list[tuple[int, str]], substitution_roots: list[str]) -> tuple[list[tuple[int, str]], list[tuple[int, str, str]]]:
    """补位那一趟（登记第一节读法 5「替换之后编不过，按 5.2 的补位规则换行」）：补位输出根里的 `row-<序号>/` 换掉前面几个输出根里同一个序号的行。
    拒（ValueError）：前面没有这一行；前面那一行的结局不是编不过；补位那一行的 `row_name` 不是前面那一行的；补位那一行编过了却没补位
    （`substituted_from` 是 `-`：原行这一次编过、上一次编不过，同一棵树上两次编的结局不同）或补位自的不是那一行的名字。交回 (换过的行表, [(序号, 原目录, 补位目录)])。"""
    by_number = dict(merged)
    replaced = []
    for number, directory in (row_directories(substitution_roots) if substitution_roots else []):
        if number not in by_number:
            raise ValueError(f"补位输出根里的第 {number} 行在前面的输出根里没有：{directory}")
        original_fields = dict(columns[:2] for columns in read_rows(os.path.join(by_number[number], "row-result.tsv")) if len(columns) >= 2)
        new_fields = dict(columns[:2] for columns in read_rows(os.path.join(directory, "row-result.tsv")) if len(columns) >= 2)
        if original_fields.get("status") != "compile_failed":
            raise ValueError(f"第 {number} 行原来的结局不是编不过（{original_fields.get('status')}），不补位：{by_number[number]}")
        if new_fields.get("row_name") != original_fields.get("row_name"):
            raise ValueError(f"第 {number} 行补位那一趟的表名与原来的不同：{new_fields.get('row_name')} 与 {original_fields.get('row_name')}")
        if new_fields.get("status") != "compile_failed" and new_fields.get("substituted_from") != original_fields.get("row_name"):
            raise ValueError(f"第 {number} 行补位那一趟没有补位自原名（substituted_from={new_fields.get('substituted_from')}）：{directory}")
        replaced.append((number, by_number[number], directory))
        by_number[number] = directory
    return sorted(by_number.items()), replaced


def row_name_for_the_result(driver_fields: dict[str, str], name_in_the_table: str) -> str:
    """结果行里这一行的名字：驱动写的 `row_name`（表里的原名，没有时用表）；补位换上的行写换上的那一行的名字（`substituted_from` 另写表里的原名）。"""
    if driver_fields.get("substituted_from", "-") != "-" and driver_fields.get("mutated_row_name", "-") != "-":
        return driver_fields["mutated_row_name"]
    return driver_fields.get("row_name", name_in_the_table)


def substitution_lines(replaced: list[tuple[int, str, str]], rows: list[RowAnalysis], lines: ResultLines) -> None:
    """补位那一趟每一行：原名、原结局、换上的行（补不上写 `none`）、试过而没用上的每一个（`<失败>:<名字>`，按先后）、换上之后这一行的分类；再一行合计。"""
    category_of = {row.number: row.category for row in rows}
    substituted = 0
    for number, original_directory, directory in replaced:
        original_fields = dict(columns[:2] for columns in read_rows(os.path.join(original_directory, "row-result.tsv")) if len(columns) >= 2)
        new_fields = dict(columns[:2] for columns in read_rows(os.path.join(directory, "row-result.tsv")) if len(columns) >= 2)
        attempts = [new_fields[f"failed_attempt_{position}"] for position in range(1, int(new_fields.get("failed_attempts", "0")) + 1)]
        substitute = new_fields.get("mutated_row_name", "-") if new_fields.get("status") != "compile_failed" else "none"
        substituted += substitute != "none"
        lines.result(f"name=substitution row={number} original_name={field(original_fields.get('row_name', '-'))} original_status={original_fields.get('status')} "
                     f"substituted_by={field(substitute)} failed_attempts={len(attempts)} attempts={field('|'.join(attempts) or 'none')} "
                     f"category={category_of.get(number, 'missing')}")
    lines.result(f"name=substitution_summary rows_that_did_not_compile={len(replaced)} substituted_rows={substituted} "
                 f"still_not_compiled={len(replaced) - substituted} row_numbers={','.join(str(number) for number, _, _ in replaced) or 'none'}")


def row_analyses(rows_roots: list[str], selected_paths: list[str], baseline_dir: str, reference_lines: list[str],
                 substitution_roots: list[str] | None = None) -> list[RowAnalysis]:
    """逐行驱动的输出根下每行一个目录 `row-<序号>/`：`row-result.tsv`（驱动写的 key<TAB>value）、`device.out`（装置标准输出）、`dump/`（装置导出目录）、`o3.out`（冻结执行器的判定）。
    `rows_roots` 可以给几份（F4 扩表那一次是 117 行那份加扩表那份），`selected_paths` 是这几份行用到的表（行名取自驱动写的 `row_name`，没有时查表）。"""
    baseline = load_run_directory(baseline_dir)
    selected = {row.number: row for path in selected_paths for row in read_selected_rows(path)}
    analyses = []
    merged, _ = rows_with_substitutes(row_directories(rows_roots), substitution_roots or [])
    for number, directory in merged:
        entry = os.path.basename(directory)
        driver_fields = {key: value for key, value in (columns[:2] for columns in read_rows(os.path.join(directory, "row-result.tsv")) if len(columns) >= 2)}
        number = int(entry.split("-")[1])
        dump = os.path.join(directory, "dump")
        run = load_run_directory(dump) if os.path.isdir(dump) else None
        device_output = os.path.join(directory, "device.out")
        summaries = device_stream_summaries(device_output) if os.path.exists(device_output) else {}
        name = row_name_for_the_result(driver_fields, selected[number].name if number in selected else entry)
        o3, reading = read_row_o3(directory, driver_fields)
        if reading is not None:  # 每份执行器产物自己带的冻结行要与冻结的参照一行不差（Z2 的另一半）
            driver_fields["o3_freeze_lines_equal_the_reference"] = str(reading.freeze_lines == reference_lines and bool(reference_lines)).lower()
            driver_fields["o3_dump_problem_lines"] = str(reading.dump_problems)
        analyses.append(analyse_row(
            number, name, driver_fields, baseline, run, summaries, o3, driver_fields.get("o3_status", "-"),
            tuple(reading.base_unrecoverable_streams) if reading else (), reading.error_states if reading else 0,
            reading.digest_mismatch_states if reading else 0))
    return analyses

def o3_baseline_lines(path: str, reference_lines: list[str], lines: ResultLines, tally: dict[str, int]) -> None:
    """A3 的 O3 一份：冻结的执行器在基线导出（批 1、批 2）上判红的态数要是 0（严格「不在」、暂代「不在」各一列），另报每态判定的分布、出错与摘要对不上的态数。"""
    reading = frozen_o3.read_executor_output(path)
    verdicts = list(reading.verdicts.values())
    strict_red = sum(pair[0] == "not_in" for pair in verdicts)
    provisional_red = sum(pair[1] == "not_in" for pair in verdicts)
    lines.result(f"name=a3_o3_baseline states={len(verdicts)} strict_in={sum(pair[0] == 'in' for pair in verdicts)} strict_not_in={strict_red} "
                 f"strict_spec_gap={sum(pair[0] == 'spec_gap' for pair in verdicts)} strict_error={sum(pair[0] == 'error' for pair in verdicts)} "
                 f"provisional_in={sum(pair[1] == 'in' for pair in verdicts)} provisional_not_in={provisional_red} "
                 f"digest_mismatch_states={reading.digest_mismatch_states} dump_problem_lines={reading.dump_problems} base_unrecoverable_streams={','.join(reading.base_unrecoverable_streams) or 'none'} "
                 f"complete={str(reading.complete).lower()} freeze_lines_equal_the_reference={str(reading.freeze_lines == reference_lines and bool(reference_lines)).lower()}")
    tally["a3_columns_with_red_on_the_baseline"] += (strict_red > 0) + (provisional_red > 0)
    tally["o3_digest_mismatch_states"] += reading.digest_mismatch_states


def q3_core_lines(items: list[str], lines: ResultLines, tally: dict[str, int]) -> None:
    """Q3-核（登记第六节）：被抓的头三行在装置上真的关掉一条（O1、O2）、或驱动不跑执行器重跑一次，其余列与不关时逐字节相同。
    每一项写成 `<O1|O2|no_executor>:<不关的那一趟的输出根>:<关了的那一趟的输出根>`，两个输出根下每个 `row-<序号>/` 逐行比：装置导出目录（耗时去掉）与执行器每态的判定（去掉耗时与进程号）。"""
    for item in items:
        closing, full_root, other_root = item.split(":", 2)
        if closing not in Q3_CORE_CLOSINGS:
            raise ValueError(f"--q3-core 写成 <{'|'.join(Q3_CORE_CLOSINGS)}>:<不关的输出根>:<关了的输出根>：{item}")
        for entry in sorted(name for name in os.listdir(other_root) if name.startswith("row-")):
            full_directory, other_directory = os.path.join(full_root, entry), os.path.join(other_root, entry)
            if not os.path.isdir(os.path.join(full_directory, "dump")):
                continue
            if closing == "no_executor":
                differences = differences_when_nothing_is_closed(os.path.join(full_directory, "dump"), os.path.join(other_directory, "dump"))
                o3_differs = "not_applicable_no_executor_in_the_closed_run"
            else:
                differences = differences_after_closing_one_oracle(os.path.join(full_directory, "dump"), os.path.join(other_directory, "dump"), closing)
                full_o3, other_o3 = (os.path.join(directory, FROZEN_EXECUTOR_OUTPUT) for directory in (full_directory, other_directory))
                if os.path.exists(full_o3) and os.path.exists(other_o3):
                    o3_differs = str(o3_state_lines_without_timing(full_o3) != o3_state_lines_without_timing(other_o3)).lower()
                else:
                    o3_differs = "o3_missing"
            tally["q3_core_runs"] += 1
            tally["q3_core_runs_with_differences"] += bool(differences) or o3_differs in ("true", "o3_missing")
            lines.result(f"name=q3_core_row closing={closing} row={entry.split('-')[1]} differences_outside_the_closed_column={len(differences)} "
                         f"o3_state_lines_differ={o3_differs} where={field(','.join(differences) or 'none')}")


def q3_core_rows(analyses: list[RowAnalysis], selected_path: str) -> list[int]:
    """Q3-核要跑的行（登记第六节）：第一刀里按表序头三行至少两条红的行；不够三行时取至少一条红的头三行。「红」取 O1、O2、O3 暂代（乙）三列。"""
    in_the_first_slice = {row.number for row in read_selected_rows(selected_path) if row.slice_number == 1}
    candidates = [row for row in sorted(analyses, key=lambda item: item.number)
                  if row.category == CATEGORY_COUNTED and row.number in in_the_first_slice and row.run is not None]
    two_or_more = [row.number for row in candidates if len(red_columns(row.run, FIRST_BATCH, row.o3, False)) >= 2]
    one_or_more = [row.number for row in candidates if len(red_columns(row.run, FIRST_BATCH, row.o3, False)) >= 1]
    return two_or_more[:Q3_CORE_ROWS] if len(two_or_more) >= Q3_CORE_ROWS else one_or_more[:Q3_CORE_ROWS]


def driver_timing_lines(path: str, lines: ResultLines) -> None:
    """逐行驱动标准输出里的 `table …` 行（整表挂钟，驱动进程里 `monotonic_ns` 计）写成耗时行，前面带 `source=<那份标准输出的文件名>`：
    `--driver-output` 可以给几份（整表分两次起、Q3-核各一次），哪一行出自哪一次驱动要分得开。口径：`table_wall_ns` 只是那一次驱动进程的挂钟；
    `--skip-finished` 的那一次把已做完的行从 `row-result.tsv` 读回、照原值并进 `rows` 与 `sum_of_row_ns`，挂钟里不含它们。不进入库产物。"""
    source = field(os.path.basename(path))
    with open(path, encoding="utf-8") as handle:
        for raw in handle:
            if raw.startswith("table "):
                lines.timing(f"name=table source={source} " + raw.rstrip("\n").removeprefix("table "))


def freeze_line(phase: str, repo_root: str, lines: ResultLines, tally: dict[str, int]) -> list[str]:
    """Z2：冻结行逐个核（开头一次、收尾一次）。交回冻结行供各份 O3 产物里的冻结行对照。"""
    reference = frozen_o3.freeze_reference_lines(repo_root)
    differences = frozen_o3.freeze_differences(repo_root, reference)
    lines.result(f"name=freeze_check phase={phase} reference={field(os.path.basename(frozen_o3.FREEZE_REFERENCE_RELATIVE))} "
                 f"files={len(frozen_o3.frozen_hashes_of(reference))} seed={frozen_o3.seed_of(reference)} differences={len(differences)} "
                 f"detail={field(','.join(differences) or 'none')}")
    tally["o3_freeze_differences"] += len(differences)
    return reference


def row_tallies(rows: list[RowAnalysis], tally: dict[str, int]) -> None:
    """判决行要的几个行数：Q1 三种读法（甲、乙、丙）各自全放行的行数、Q3 两种读法各自只靠一条的行数、执行器没跑完的行数、摘要对不上的态数。"""
    counted = counted_rows(rows)
    o3_ran = any(row.o3 is not None for row in rows)
    if o3_ran:
        for label, oracles in Q1_READINGS:
            tally[f"{label}_all_quiet_rows"] = len(quiet_rows_in_batch(counted, oracles, FIRST_BATCH))
        for label, strict in (("q3_alpha", True), ("q3_beta", False)):
            tally[f"{label}_rows_held_by_a_single_oracle"] = sum(
                bool(oracles_that_alone_hold_the_row(red_columns(row.run, FIRST_BATCH, row.o3, strict))) for row in counted if row.run is not None)
        tally["q1_o3_strict_reading_unreachable"] = int(strict_reading_is_unreachable(
            sum(row.run is not None and o3_judged_state_count(row.run, row.o3, 0) == 0 for row in counted), len(counted)))
    tally["rows_with_an_errored_call_the_comparison_did_not_judge"] = sum(
        any(errored_calls_left_unjudged(row.run).values()) for row in rows if row.run is not None)
    tally["rows_with_o1_not_applicable_from_another_source"] = sum(
        any(not_applicable_states_with_another_source(row.run).values()) for row in rows if row.run is not None)
    tally["rows_with_the_o3_executor_failed"] = sum(row.o3_status == O3_STATUS_EXECUTOR_FAILED for row in rows)
    tally["o3_digest_mismatch_states"] += sum(row.o3_digest_mismatch_states for row in rows)
    tally["o3_dump_problem_lines"] = sum(int(row.driver_fields.get("o3_dump_problem_lines", "0")) for row in rows)
    tally["rows_with_o3_freeze_lines_not_the_reference"] = sum(row.driver_fields.get("o3_freeze_lines_equal_the_reference") == "false" for row in rows)
    # 登记修订 5：编过的行（驱动记 status=ok）都要带驱动编完核过「这一行与上一行改过的包都重编了」的那个键；没带的是旧驱动跑的，可能带着上一行的变异
    tally["rows_compiled_without_the_rebuild_check"] = sum(
        row.driver_fields.get("status") == STATUS_OK and REBUILD_CHECK_FIELD not in row.driver_fields for row in rows)


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E171 比对器")
    parser.add_argument("--output")
    parser.add_argument("--mutations-table")
    parser.add_argument("--selected-rows", default=os.path.join(HERE, "bad-implementations-117.tsv"))
    parser.add_argument("--baseline-dir")
    parser.add_argument("--baseline-device-output")
    parser.add_argument("--baseline-repeat-dir")
    parser.add_argument("--e170-dump")
    parser.add_argument("--o3-baseline")
    parser.add_argument("--driver-output", action="append", default=[])
    parser.add_argument("--control", action="append", default=[])
    parser.add_argument("--disabled", action="append", default=[])
    parser.add_argument("--q3-core", action="append", default=[])
    parser.add_argument("--rows-root", action="append", default=[])
    parser.add_argument("--expansion-rows", help="F4 扩表（bad-implementations-expansion-164.tsv）")
    parser.add_argument("--substitution-rows-root", action="append", default=[],
                        help="补位那一趟的输出根：里面的行换掉 --rows-root 里同一个序号、编不过的那一行（登记第一节读法 5）")
    parser.add_argument("--segment", default="s4", help="config 行里的段名")
    parser.add_argument("--known-bad-dir")
    parser.add_argument("--e170-expected-dir")
    parser.add_argument("--image-judgements")
    parser.add_argument("--print-q3-core-rows", action="store_true")
    arguments = parser.parse_args(argument_list)
    repo_root = os.path.abspath(os.path.join(HERE, "../../.."))
    if arguments.print_q3_core_rows:
        analyses = row_analyses(arguments.rows_root, [arguments.selected_rows], arguments.baseline_dir, [])
        print("q3_core_rows=" + ",".join(map(str, q3_core_rows(analyses, arguments.selected_rows))))
        return 0
    given = {"output": arguments.output, "mutations_table": arguments.mutations_table, "baseline_dir": arguments.baseline_dir,
             "baseline_device_output": arguments.baseline_device_output}  # 逐个点名读，不按选项名 getattr
    for required, value in given.items():
        if value is None:
            parser.error(f"--{required.replace('_', '-')} 必给")
    if frozen_o3.report_missing_freeze_reference(repo_root, "comparator.py"):   # 冻结行的参照产物不在：本次未跑，一行产物都不写
        return frozen_o3.NOT_RUN_EXIT_CODE
    lines = ResultLines(arguments.output)
    started = time.monotonic_ns()
    o3_columns = "frozen_executor" if (arguments.o3_baseline or arguments.rows_root or arguments.image_judgements) else "not_run"
    lines.result(f"name=config segment={field(arguments.segment)} o3={o3_columns} baseline_run={field(os.path.basename(arguments.baseline_dir))} "
                 f"rows_root={field('+'.join(os.path.basename(root) for root in arguments.rows_root) or 'none')} "
                 f"controls={len(arguments.control)} disabled_runs={len(arguments.disabled)} q3_core_comparisons={len(arguments.q3_core)}"
                 + (f" substitution_rows_root={field('+'.join(os.path.basename(root) for root in arguments.substitution_rows_root))}"
                    if arguments.substitution_rows_root else ""))
    tally = collections.Counter()
    reference = freeze_line("start", repo_root, lines, tally)
    registered_table_lines(arguments.mutations_table, arguments.selected_rows, lines)
    if arguments.expansion_rows:
        expansion_table_lines(arguments.mutations_table, arguments.selected_rows, arguments.expansion_rows, lines)
    baseline_lines(arguments.baseline_dir, arguments.baseline_device_output, arguments.baseline_repeat_dir, arguments.e170_dump, lines, tally)
    if arguments.o3_baseline:
        o3_baseline_lines(arguments.o3_baseline, reference, lines, tally)
    control_lines(arguments.control, lines, tally)
    disabled_lines(arguments.disabled, arguments.baseline_dir, lines, tally)
    if arguments.rows_root:
        selected_paths = [arguments.selected_rows] + ([arguments.expansion_rows] if arguments.expansion_rows else [])
        rows = row_analyses(arguments.rows_root, selected_paths, arguments.baseline_dir, reference, arguments.substitution_rows_root)
        for row in rows:
            row_lines(row, lines)
        q1_lines(rows, lines, o3_available=any(row.o3 is not None for row in rows), baseline=load_run_directory(arguments.baseline_dir))
        if arguments.expansion_rows:
            q1_denominator_by_table_lines(rows, {row.number for row in read_selected_rows(arguments.expansion_rows)}, lines)
        if arguments.substitution_rows_root:
            substitution_lines(rows_with_substitutes(row_directories(arguments.rows_root), arguments.substitution_rows_root)[1], rows, lines)
        timing_lines(rows, lines)
        tally["rows_untouched_but_red"] = len(untouched_but_red_rows(rows))
        row_tallies(rows, tally)
    q3_core_lines(arguments.q3_core, lines, tally)
    if arguments.known_bad_dir and arguments.e170_expected_dir:
        bad_rows = read_known_bad_rows(arguments.known_bad_dir) + read_e170_rows(arguments.known_bad_dir, arguments.e170_expected_dir)
        judgements = read_image_judgements(arguments.image_judgements) if arguments.image_judgements else None
        if judgements is not None:  # Z4 的坏镜像一半：执行器建的镜像（补丁之前与之后两个摘要）与导出写的是不是同一个状态
            mismatches = sum(not (found.base_digest_ok and found.patched_digest_ok) for found in judgements.values())
            lines.result(f"name=o3_image_digest_check images={len(judgements)} digest_mismatches={mismatches}")
            tally["o3_image_digest_mismatches"] = mismatches
        base_judgements = None if judgements is None else {
            (stream, label): found for (source, stream, _, label), found in judgements.items() if source == SOURCE_BASE}
        bad_image_lines(attach_o3(bad_rows, judgements), base_judgements, lines, tally)
    freeze_line("end", repo_root, lines, tally)
    for driver_output in arguments.driver_output:
        driver_timing_lines(driver_output, lines)
    lines.timing(f"name=comparator_wall_ns ns={time.monotonic_ns() - started}")
    lines.result("name=verdict " + " ".join(f"{key}={tally[key] if key in tally else 'not_run'}" for key in VERDICT_KEYS))
    lines.finish()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
