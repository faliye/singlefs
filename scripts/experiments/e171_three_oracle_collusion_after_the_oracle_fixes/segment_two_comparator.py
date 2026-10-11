"""E171 第 1 次重跑第二段（整表 281 行加补位，流一、流二，批 1 批 2，不跑执行器、不带探针）的比对器：登记第六节 Q2-集合、Q2-判、Q2-观测、
Q2-七行（整表上照报）、Q3-分母（D、原登记读法 5 的算法、批 2 的算法、各类逐行），第八节「Q3 的分母」两批并列与第 125 行两批计不计入，
作废 Z6 的 `rows_compiled_without_the_rebuild_check`、停机 S5 的比对器一侧两个行数、修订 5 第 5.1 节「已有产物的读法」的恢复 panic 数法。

另写一份、不改第一段比对器：第一段判决、单测与变异表都不许变，第二段的输入（一份基线、一个行输出根、一份逐行驱动合成产物）与第一段不同。
谓词照用 `collusion_matrix.py`（触达、交回值差、可判调用数、自崩、S5 两数）；这里只多批 2 上的触达、分类与汇总，都是纯函数，由 `test_segment_two.py` 单测。

基线：修订 6 树上 S2 那一趟新装置基线（`--streams 1,2`、不带探针；登记修订 6 第 6.1 节末表那一格说 Q2 的集合拿第 1 步这一趟比，
修订 2 第 2.5 节第二段的 T 也出自它）。读法 4「基线连跑两次摘要不同的对象从比较里拿掉」：第二段那一趟基线只跑了一次，`--stability-baseline`
给第一段两次三流带探针的基线（同一棵树、同一份装置），只比流一、流二，各自与基线比出的不稳对象并起来拿掉；八个文件逐个比的结果照报。

只读：基线与稳定性基线的导出目录、行输出根（`row-<序号>/row-result.tsv` 与 `dump/`）、逐行驱动的合成产物、冻结树里的变异表；写一份新产物（已存在就拒绝）。

跑法（在本目录）：
  python3 segment_two_comparator.py --baseline <导出目录> [--stability-baseline <导出目录> …] --rows-root <行输出根> --driver-product <合成产物>
      [--mutations-table <冻结树的 crates/mutations.tsv>] --dumps-root <导出根> [--dumps-root <导出根> …] --product-prefix <前缀>
      --product-suffix=<后缀（以 - 起头，写成 = 连着）> --output <新文件>
导出目录不在给的任一导出根之下、合成产物名不带这一趟的前缀与后缀的，退 2、不写产物（同第一段比对器，登记修订 8 第 8.5 节第 1 条）。
"""

from __future__ import annotations

import argparse
import dataclasses
import os
import sys
from dataclasses import dataclass

import collusion_matrix as matrix
import segment_one_comparator as segment_one
from result_lines import ResultLines, field
from row_driver import REBUILD_CHECK_FIELD
from run_directory import TOUCH_STREAM_NAMES, RunDirectory, file_sha256, load_run_directory, result_lines_named

TABLE_ROWS = 281  # 117 + 164（登记第七节 A6）
STATUS_OK = "ok"
RAN_EXIT_CODE = "0"
DEVICE_STOP_EXIT_CODE = "3"  # 装置自己停机（S 类回比不过）：原比对器 `collusion_report.classify_row` 的 `device_stop`，不进分母
TIMEOUT_EXIT_CODE = "253"  # run-with-memory-cap.sh 的超时
WRAPPER_EXIT_CODES = ("250", "251", "252", "254")  # 包装自己的其余结局（作废 Z3）
NO_RECOVERY_PANIC = "-"

CATEGORY_COUNTED = "counted"
CATEGORY_UNTOUCHED = "untouched"
CATEGORY_ONLY_IN_THE_IGNORED_PASS = "only_in_the_ignored_pass"
CATEGORY_SELF_CRASH = "self_crash"
CATEGORY_TIMEOUT = "timeout"
CATEGORY_WRAPPER_ENDED = "wrapper_ended"
CATEGORY_DEVICE_STOP = "device_stop"
# 结果行里各类的次序：先进分母的、再两类未触达、再原登记读法 5 那几类（逐行驱动的四种定位失败照它写的 status 名）、再装置这一侧的结局
CATEGORY_ORDER = (CATEGORY_COUNTED, CATEGORY_UNTOUCHED, CATEGORY_ONLY_IN_THE_IGNORED_PASS, CATEGORY_SELF_CRASH, "compile_failed",
                  "anchor_not_unique", "ambiguous_name", "not_found", CATEGORY_TIMEOUT, CATEGORY_WRAPPER_ENDED, CATEGORY_DEVICE_STOP)

# 读法 5 的两种算法：登记（第 1 次重跑读法 5 收窄过的「实现自崩」）与原登记（调用或态上看 journal 那遍恢复里 panic 一律自崩）
READING_REGISTRATION = "registration"
READING_ORIGINAL = "original_reading_5"
READINGS = (READING_REGISTRATION, READING_ORIGINAL)
BATCHES = (matrix.FIRST_BATCH, matrix.SECOND_BATCH)
# 基线与稳定性基线逐个比的文件（流一、流二各自）
STABILITY_FILES = ("ops.tsv", "ops-payload.bin", "calls.tsv", "base-sectors.bin", "states.tsv", "oracle-verdicts.tsv", "call-outcomes.tsv",
                   "call-return-values.tsv")


def with_only_the_batch_as_the_first(run: RunDirectory, batch: int) -> RunDirectory:
    """只留 `batch` 那一批的取样态、批号改记成批 1 的副本：把它交给 `matrix.touch_report`，读法 4 的第三样就换在这一批上比，
    别的两样（写流摘要、调用结局）不看取样态、不变。批 1 时与原样交给 `matrix.touch_report` 同值（`test_segment_two.py` 对拍）。"""
    streams = {name: dataclasses.replace(stream, states=[dataclasses.replace(state, batch=matrix.FIRST_BATCH)
                                                         for state in stream.states if state.batch == batch])
               for name, stream in run.streams.items()}
    return RunDirectory(run.path, streams)


def touch_report_on_the_batch(baseline: RunDirectory, row: RunDirectory, batch: int) -> matrix.TouchReport:
    """读法 4 的触达（原登记一个字不改），第三样用 `batch` 那一批的取样态比（第八节「Q3 的分母」：批 1 定触达、批 2 定触达两个 D 并列）。"""
    return matrix.touch_report(with_only_the_batch_as_the_first(baseline, batch), with_only_the_batch_as_the_first(row, batch))


def recovery_panic_states(run: RunDirectory) -> int:
    """流一、流二两批取样态里看 journal 那遍恢复 panic 的态数（`oracle-verdicts.tsv` 恢复 panic 一列不是 `-`；修订 5 第 5.1 节第 5 条之后
    那一列只记看 journal 那一遍）。"""
    return sum(state.recovery_panic != NO_RECOVERY_PANIC for name in TOUCH_STREAM_NAMES for state in run.streams[name].states)


def is_a_self_crash_by_the_original_reading_5(run: RunDirectory) -> bool:
    """原登记读法 5：驱动一次调用或某一态的看 journal 恢复里 panic，一律是实现自崩、不进分母（流一、流二；两批的取样态都算）。"""
    call_panicked = any(call.panicked() for name in TOUCH_STREAM_NAMES for call in run.streams[name].calls)
    return call_panicked or recovery_panic_states(run) > 0


def not_ran_category(driver_fields: dict[str, str]) -> str | None:
    """驱动没造出可判的一趟时这一行的类：定位失败照驱动写的 status（编不过、锚点不唯一、名字不唯一、找不到）；装置那一步超时、包装的别的结局、
    装置自己停机各一类；别的非 0 退出码记 `device_exit_<码>`。造出了可判的一趟交回 None。"""
    status = driver_fields.get("status", "missing")
    if status != STATUS_OK:
        return status
    exit_code = driver_fields.get("device_exit", "-")
    if exit_code == TIMEOUT_EXIT_CODE:
        return CATEGORY_TIMEOUT
    if exit_code in WRAPPER_EXIT_CODES:
        return CATEGORY_WRAPPER_ENDED
    if exit_code == DEVICE_STOP_EXIT_CODE:
        return CATEGORY_DEVICE_STOP
    if exit_code != RAN_EXIT_CODE:
        return f"device_exit_{exit_code}"
    return None


def ran_category(touch: matrix.TouchReport, self_crash: bool) -> str:
    """跑成了的一行：实现自崩（按所取的读法）不进分母；否则触达进分母；未触达再分「只在不看 journal 那一遍不同」与未触达（原登记读法 4）。"""
    if self_crash:
        return CATEGORY_SELF_CRASH
    if touch.touched():
        return CATEGORY_COUNTED
    return CATEGORY_ONLY_IN_THE_IGNORED_PASS if touch.ignored_pass_differs else CATEGORY_UNTOUCHED


@dataclass(frozen=True)
class RowReading:
    """整表一行：驱动记的字段、不看读法与批就定下的类（没造出可判的一趟，或流一流二在第一次调用之前就 panic）、读回的导出（只有流一、流二）、
    两批各自的触达、两种读法各自是不是实现自崩。"""

    number: int
    driver_fields: dict[str, str]
    fixed_category: str | None
    run: RunDirectory | None
    touch_by_batch: dict[int, matrix.TouchReport]
    self_crash_by_reading: dict[str, bool]

    def category(self, reading: str, batch: int) -> str:
        if self.fixed_category is not None:
            return self.fixed_category
        return ran_category(self.touch_by_batch[batch], self.self_crash_by_reading[reading])

    def untouched(self) -> bool:
        """读法 4 未触达（批 1 定判，第五节 5.1 Q2 那一行）；没有导出可比的一行不算未触达。"""
        return self.fixed_category is None and not self.touch_by_batch[matrix.FIRST_BATCH].touched()

    def not_located(self) -> bool:
        """逐行驱动没找到可施加的变异（找不到、名字不唯一、锚点不唯一、补位之后仍编不过）：读法 5 的定位失败那几类。"""
        return self.driver_fields.get("status", "missing") != STATUS_OK


PIPELINE_PANIC_PREFIXES = ("pipeline_panicked", "implementation_crashed")  # 装置 `name=stream` 行 `not_exported_why=` 的前缀：流没导出是因为 panic


def panicked_before_the_first_call(stream_lines: list[dict[str, str]]) -> bool:
    """流一、流二都没导出、都是 panic、都一次调用没发（`calls=0`）：mkfs 那一步就 panic 了。这样的行没有导出可比，O1-对拍判过的调用为 0，
    两种读法都记实现自崩（第 1 次重跑读法 5「自崩前可判的调用数为 0」；原登记读法 5 一律自崩）。"""
    by_stream = {line.get("stream"): line for line in stream_lines}
    return all(name in by_stream and by_stream[name].get("exported") == "false"
               and by_stream[name].get("not_exported_why", "-").startswith(PIPELINE_PANIC_PREFIXES) and by_stream[name].get("calls") == "0"
               for name in TOUCH_STREAM_NAMES)


def reading_of(number: int, driver_fields: dict[str, str], run: RunDirectory | None, baseline: RunDirectory,
               stream_lines: list[dict[str, str]] | None = None) -> RowReading:
    """`stream_lines` 是装置标准输出里的 `name=stream` 行（没导出时用来判 mkfs 那一步的 panic）。"""
    fixed_category = not_ran_category(driver_fields)
    if fixed_category is None and run is None:
        fixed_category = CATEGORY_SELF_CRASH if panicked_before_the_first_call(stream_lines or []) else "dump_missing"
    if fixed_category is not None:
        return RowReading(number, driver_fields, fixed_category, None, {}, {})
    assert run is not None
    touch_by_batch = {batch: touch_report_on_the_batch(baseline, run, batch) for batch in BATCHES}
    self_crash_by_reading = {READING_REGISTRATION: matrix.counts_as_a_self_crash(run),
                             READING_ORIGINAL: is_a_self_crash_by_the_original_reading_5(run)}
    return RowReading(number, driver_fields, None, run, touch_by_batch, self_crash_by_reading)


def self_crash_place(row: RowReading) -> str:
    """自崩的那一行 panic 在哪：mkfs 那一步（`before_the_first_call`）；流一、流二各自的第一处调用 panic（`<流>:call_<序号>:judged_<之前可判的调用数>`）；
    态上看 journal 那遍恢复 panic（`recovery_panic_states_<态数>`）。几处都有的用逗号连起来。"""
    if row.run is None:
        return "before_the_first_call"
    places = []
    for name in TOUCH_STREAM_NAMES:
        calls = row.run.streams[name].calls
        panic_at = next((call.index for call in calls if call.panicked()), None)
        if panic_at is not None:
            places.append(f"{name}:call_{panic_at}:judged_{sum(call.was_judged_by_the_comparison() for call in calls[:panic_at])}")
    states = recovery_panic_states(row.run)
    if states:
        places.append(f"recovery_panic_states_{states}")
    return ",".join(places) or "-"


def denominator(rows: list[RowReading], reading: str, batch: int) -> int:
    """Q3-分母 D：触达且不在读法 5 那几类里的行数（按所取的读法与定触达的那一批）。"""
    return sum(row.category(reading, batch) == CATEGORY_COUNTED for row in rows)


def rows_by_category(rows: list[RowReading], reading: str, batch: int) -> dict[str, list[int]]:
    """各类逐行：类 → 行号（升序）。类的次序照 `CATEGORY_ORDER`，表外的类排在后面、按名字。"""
    found: dict[str, list[int]] = {}
    for row in sorted(rows, key=lambda item: item.number):
        found.setdefault(row.category(reading, batch), []).append(row.number)
    ordered = [category for category in CATEGORY_ORDER if category in found] + sorted(set(found) - set(CATEGORY_ORDER))
    return {category: found[category] for category in ordered}


@dataclass(frozen=True)
class Q2Row:
    """Q2 集合里的一行（或点名七行之一）：每次与基线不同的调用（交回值顶层字段、内存对象、胶水造出的观测同不同）与 O1-对拍红的调用数。"""

    number: int
    differences: tuple[matrix.ReturnValueDifference, ...]
    comparison_red_calls: int

    def observation_differs_calls(self) -> int:
        return sum(difference.observation_differs for difference in self.differences)


def q2_row_of(row: RowReading, baseline: RunDirectory, unstable: frozenset[str]) -> Q2Row:
    assert row.run is not None
    differences = tuple(matrix.return_value_differences(baseline, row.run, unstable, TOUCH_STREAM_NAMES))
    return Q2Row(row.number, differences, sum(matrix.red_calls(row.run, TOUCH_STREAM_NAMES).values()))


def q2_set(rows: list[RowReading], baseline: RunDirectory, unstable: frozenset[str]) -> list[Q2Row]:
    """Q2-集合：整表里按读法 4 未触达、而至少一次调用的交回值或内存对象摘要与基线不同的行（流一、流二）。"""
    found = []
    for row in sorted(rows, key=lambda item: item.number):
        if not row.untouched():
            continue
        q2_row = q2_row_of(row, baseline, unstable)
        if q2_row.differences:
            found.append(q2_row)
    return found


def q2_verdict(set_rows: int, red_rows: int) -> str:
    """Q2-判：集合空走 F4（不够判）；r = n 甲、0 < r < n 乙、r = 0 丙。"""
    if set_rows == 0:
        return "not_enough_f4"
    if red_rows == set_rows:
        return "jia"
    return "yi" if red_rows > 0 else "bing"


def model_word_of_the_call(run: RunDirectory, stream: str, call_index: int) -> str:
    """那次调用 O1-对拍一栏记的词（`call-outcomes.tsv` 第 6 列）；调用表里没有这次调用写 `-`。"""
    return next((call.model_word for call in run.streams[stream].calls if call.index == call_index), "-")


def label_of_the_call(run: RunDirectory, stream: str, call_index: int) -> str:
    return next((call.label for call in run.streams[stream].calls if call.index == call_index), "-")


def driver_product_lines(path: str) -> tuple[dict[str, dict[str, str]], dict[int, dict[str, str]]]:
    """逐行驱动合成产物：开头几种行（`name=row_time_limit`、`snapshot_digest=`、`table`，按首个词分）与每行一条 `row row_number=…`。
    值里的空白驱动已换成下划线，按空格切字段。"""
    headers: dict[str, dict[str, str]] = {}
    rows: dict[int, dict[str, str]] = {}
    with open(path, encoding="utf-8") as handle:
        for raw in handle:
            tokens = raw.rstrip("\n").split(" ")
            fields = dict(token.partition("=")[::2] for token in tokens if "=" in token)
            if tokens[0] == "row":
                rows[int(fields["row_number"])] = fields
            elif tokens[0] == "name=row_time_limit":
                headers["row_time_limit"] = fields
            elif tokens[0].startswith("snapshot_digest="):
                headers["snapshot"] = fields
            elif tokens[0] == "table":
                headers["table"] = fields
    return headers, rows


def row_numbers_in(rows_root: str) -> list[int]:
    return sorted(int(name.removeprefix("row-")) for name in os.listdir(rows_root)
                  if name.startswith("row-") and name.removeprefix("row-").isdigit())


def read_the_row(baseline: RunDirectory, rows_root: str, number: int) -> RowReading:
    row_directory = os.path.join(rows_root, f"row-{number}")
    driver_fields = segment_one.read_row_result(row_directory)
    dump = os.path.join(row_directory, "dump")
    run = load_run_directory(dump) if os.path.isdir(dump) else None
    device_output = os.path.join(row_directory, "device.out")
    stream_lines = result_lines_named(device_output, "stream") if run is None and os.path.exists(device_output) else []
    return reading_of(number, driver_fields, run, baseline, stream_lines)


def name_hits_in_the_table(mutations_table: str | None, name: str) -> str:
    """一个变异名在冻结树的 `crates/mutations.tsv` 第 1 列出现几次（找不到那一类为什么补不上的佐证）；没给表写 `-`。"""
    if mutations_table is None:
        return "-"
    with open(mutations_table, encoding="utf-8") as handle:
        return str(sum(line.split("\t", 1)[0] == name for line in handle if not line.startswith("#")))


def stability_lines(lines: ResultLines, baseline_path: str, baseline: RunDirectory, others: list[tuple[str, RunDirectory]]) -> tuple[frozenset[str], int]:
    """读法 4 的不稳对象：基线与每份稳定性基线在流一、流二上比，摘要不同的内存对象并起来；八个文件逐个比。交回 (不稳对象, 不同的文件数)。"""
    unstable: set[str] = set()
    differing_files = 0
    for other_path, other in others:
        for stream in TOUCH_STREAM_NAMES:
            same = []
            for name in STABILITY_FILES:
                left, right = os.path.join(baseline_path, stream, name), os.path.join(other_path, stream, name)
                same.append(os.path.exists(left) and os.path.exists(right) and file_sha256(left) == file_sha256(right))
            differing_files += same.count(False)
            lines.result(f"name=baseline_stability other={field(other_path)} stream={stream} files={len(STABILITY_FILES)} identical={sum(same)} "
                         f"differing={field(','.join(name for name, flag in zip(STABILITY_FILES, same) if not flag) or '-')}")
        # 基线只有流一、流二（流三目录不在、交回值表为空），`unstable_memory_objects` 按基线的调用逐个找对面，流三自然不进来
        unstable |= matrix.unstable_memory_objects(baseline, other)
    lines.result(f"name=unstable_memory_objects stability_baselines={len(others)} names={field(','.join(sorted(unstable)) or '-')}")
    return frozenset(unstable), differing_files


def row_line(lines: ResultLines, row: RowReading) -> None:
    """一行的分类与触达三样（两批）、调用 panic 与可判调用数、恢复 panic 态数，四种「读法 × 批」各一个类。"""
    fields = row.driver_fields
    head = (f"name=row row={row.number} status={field(fields.get('status', 'missing'))} device_exit={field(fields.get('device_exit', '-'))} "
            f"substituted_from={field(fields.get('substituted_from', '-'))} failed_attempts={field(fields.get('failed_attempts', '-'))} "
            f"mutated_row_name={field(fields.get('mutated_row_name', '-'))}")
    categories = " ".join(f"category_{reading}_batch_{batch}={row.category(reading, batch)}" for reading in READINGS for batch in BATCHES)
    if row.run is None:
        # 没有导出的行：定位失败、装置那一步的结局（超时、包装、停机），或流一流二在第一次调用之前就 panic（这一种单标出来）
        before_the_first_call = not row.not_located() and row.fixed_category == CATEGORY_SELF_CRASH
        lines.result(f"{head} ran=false panicked_before_the_first_call={str(before_the_first_call).lower()} {categories}")
        return
    first, second = row.touch_by_batch[matrix.FIRST_BATCH], row.touch_by_batch[matrix.SECOND_BATCH]
    judged = matrix.judged_calls_before_the_first_panic(row.run)
    lines.result(f"{head} ran=true write_stream_differs={str(first.write_stream_differs).lower()} call_outcomes_differ={str(first.call_outcomes_differ).lower()} "
                 f"recovery_report_differs_batch_1={str(first.recovery_report_differs).lower()} recovery_report_differs_batch_2={str(second.recovery_report_differs).lower()} "
                 f"ignored_pass_differs_batch_1={str(first.ignored_pass_differs).lower()} ignored_pass_differs_batch_2={str(second.ignored_pass_differs).lower()} "
                 f"call_panic={str(judged is not None).lower()} judged_calls_before_the_first_panic={'-' if judged is None else judged} "
                 f"recovery_panic_states={recovery_panic_states(row.run)} {categories}")


def q2_call_lines(lines: ResultLines, line_name: str, row: RowReading, q2_row: Q2Row) -> None:
    """一行里每次与基线不同的调用：哪条流第几次调用（标签）、交回值哪几个顶层字段不同、哪个内存对象不同、胶水交给模型的观测同不同、O1-对拍记的词。"""
    assert row.run is not None
    for difference in q2_row.differences:
        lines.result(f"name={line_name} row={row.number} stream={difference.stream} call={difference.call_index} "
                     f"label={field(label_of_the_call(row.run, difference.stream, difference.call_index))} "
                     f"fields={field(','.join(difference.differing_fields) or '-')} objects={field(','.join(difference.differing_objects) or '-')} "
                     f"observation_differs={str(difference.observation_differs).lower()} "
                     f"o1_comparison={field(model_word_of_the_call(row.run, difference.stream, difference.call_index))}")


def q2_lines(lines: ResultLines, rows: list[RowReading], baseline: RunDirectory, unstable: frozenset[str]) -> tuple[str, list[Q2Row]]:
    """Q2-集合（逐行逐次调用）、Q2-判、Q2-观测、Q2-七行（整表上，不论在不在集合里）。交回 (判定, 集合)。"""
    found = q2_set(rows, baseline, unstable)
    by_number = {row.number: row for row in rows}
    for q2_row in found:
        lines.result(f"name=q2_set_row row={q2_row.number} calls_with_a_difference={len(q2_row.differences)} "
                     f"calls_whose_observation_differs={q2_row.observation_differs_calls()} "
                     f"calls_whose_observation_is_the_same={len(q2_row.differences) - q2_row.observation_differs_calls()} "
                     f"o1_comparison_red_calls={q2_row.comparison_red_calls} o1_red={str(q2_row.comparison_red_calls > 0).lower()}")
        q2_call_lines(lines, "q2_set_call", by_number[q2_row.number], q2_row)
    red_rows = [q2_row.number for q2_row in found if q2_row.comparison_red_calls > 0]
    quiet_rows = [q2_row.number for q2_row in found if q2_row.comparison_red_calls == 0]
    verdict = q2_verdict(len(found), len(red_rows))
    differing_calls = sum(len(q2_row.differences) for q2_row in found)
    observation_differs = sum(q2_row.observation_differs_calls() for q2_row in found)
    lines.result(f"name=q2_observation set_rows={len(found)} calls_with_a_difference={differing_calls} calls_whose_observation_differs={observation_differs} "
                 f"calls_whose_observation_is_the_same={differing_calls - observation_differs}")
    set_numbers = {q2_row.number for q2_row in found}
    for number in segment_one.Q2_NAMED_ROWS:
        row = by_number.get(number)
        if row is None or row.run is None:
            lines.result(f"name=q2_named_row row={number} ran=false category={'missing' if row is None else row.fixed_category}")
            continue
        q2_row = q2_row_of(row, baseline, unstable)
        lines.result(f"name=q2_named_row row={number} ran=true touched={str(not row.untouched()).lower()} in_the_set={str(number in set_numbers).lower()} "
                     f"calls_with_a_difference={len(q2_row.differences)} calls_whose_observation_differs={q2_row.observation_differs_calls()} "
                     f"calls_whose_observation_is_the_same={len(q2_row.differences) - q2_row.observation_differs_calls()} "
                     f"o1_comparison_red_calls={q2_row.comparison_red_calls}")
        q2_call_lines(lines, "q2_named_row_call", row, q2_row)
    lines.result(f"name=q2 set_rows={len(found)} o1_red_rows={len(red_rows)} rows_o1_red={field(','.join(map(str, red_rows)) or '-')} "
                 f"rows_o1_not_red={field(','.join(map(str, quiet_rows)) or '-')} verdict={verdict}")
    return verdict, found


def q3_lines(lines: ResultLines, rows: list[RowReading]) -> dict[str, int]:
    """Q3-分母：两种读法 × 两批，各一行 D 与各类行数，各类逐行一行；第 125 行四格计不计入。交回 {读法_批: D}。"""
    denominators = {}
    for reading in READINGS:
        for batch in BATCHES:
            grouped = rows_by_category(rows, reading, batch)
            counted = denominator(rows, reading, batch)
            denominators[f"{reading}_batch_{batch}"] = counted
            lines.result(f"name=q3_denominator reading={reading} batch={batch} rows={len(rows)} counted_D={counted} "
                         + " ".join(f"{category}={len(numbers)}" for category, numbers in grouped.items()))
            for category, numbers in grouped.items():
                lines.result(f"name=q3_denominator_class reading={reading} batch={batch} class={category} rows={len(numbers)} "
                             f"row_numbers={','.join(map(str, numbers))}")
    for row in rows:
        if CATEGORY_SELF_CRASH in (row.category(READING_REGISTRATION, matrix.FIRST_BATCH), row.category(READING_ORIGINAL, matrix.FIRST_BATCH)):
            lines.result(f"name=q3_self_crash_row row={row.number} where={self_crash_place(row)} "
                         f"registration={row.category(READING_REGISTRATION, matrix.FIRST_BATCH)} original_reading_5={row.category(READING_ORIGINAL, matrix.FIRST_BATCH)}")
    row = next((item for item in rows if item.number == segment_one.Q3_ROW), None)
    if row is None:
        lines.result(f"name=q3_row_125 row={segment_one.Q3_ROW} ran=false category=missing")
        return denominators
    lines.result(f"name=q3_row_125 row={row.number} "
                 + " ".join(f"{reading}_batch_{batch}={row.category(reading, batch)} counted_{reading}_batch_{batch}="
                            f"{str(row.category(reading, batch) == CATEGORY_COUNTED).lower()}" for reading in READINGS for batch in BATCHES))
    return denominators


def check_lines(lines: ResultLines, rows: list[RowReading], product_rows: dict[int, dict[str, str]], mutations_table: str | None) -> dict[str, object]:
    """作废 Z6、停机 S5 的比对器一侧、修订 5 第 5.1 节的恢复 panic 数法、找不到与编不过那几行（补位）、未触达而有一列红的行、行目录与合成产物对账。"""
    ran = [row for row in rows if row.run is not None]
    checks: dict[str, object] = {
        # 作废 Z6：编过的行都要带驱动编完核过重编的那个键
        "rows_compiled_without_the_rebuild_check": sum(row.driver_fields.get("status") == STATUS_OK and REBUILD_CHECK_FIELD not in row.driver_fields
                                                       for row in rows),
        "rows_with_an_errored_call_the_comparison_did_not_judge": sum(any(matrix.errored_calls_left_unjudged(row.run).values()) for row in ran if row.run),
        "rows_with_o1_not_applicable_from_another_source": sum(any(matrix.not_applicable_states_with_another_source(row.run).values())
                                                               for row in ran if row.run),
    }
    panicking = [(row.number, recovery_panic_states(row.run)) for row in ran if row.run and recovery_panic_states(row.run)]
    lines.result(f"name=recovery_panic_sum rows={len(panicking)} states={sum(states for _, states in panicking)} "
                 f"by_row={field(','.join(f'{number}:{states}' for number, states in panicking) or '-')}")
    for row in rows:
        if not row.not_located():
            continue
        name = row.driver_fields.get("row_name", "-")
        lines.result(f"name=not_located_row row={row.number} status={row.fixed_category} failed_attempts={field(row.driver_fields.get('failed_attempts', '-'))} "
                     f"substituted_from={field(row.driver_fields.get('substituted_from', '-'))} name_hits_in_the_frozen_table={name_hits_in_the_table(mutations_table, name)}")
    for row in rows:
        if row.driver_fields.get("substituted_from", "-") != "-":
            lines.result(f"name=substitution row={row.number} status={field(row.driver_fields.get('status', '-'))} "
                         f"failed_attempts={field(row.driver_fields.get('failed_attempts', '-'))} substituted_by={field(row.driver_fields.get('mutated_row_name', '-'))}")
    untouched_red = []
    for row in ran:
        if not row.untouched() or row.run is None:
            continue
        reds = {"O1-对拍": sum(matrix.red_calls(row.run, TOUCH_STREAM_NAMES).values())}
        reds.update({column: sum(matrix.red_state_counts(row.run, column, matrix.FIRST_BATCH, None, TOUCH_STREAM_NAMES).values())
                     for column in (matrix.SUBCOLUMN_CRASH_JUDGE, matrix.SUBCOLUMN_REFERENCE_FILESYSTEM, matrix.ORACLE_TWO)})
        if any(reds.values()):
            untouched_red.append(row.number)
            lines.result(f"name=untouched_row_red row={row.number} " + " ".join(f"{field(column)}={count}" for column, count in reds.items()))
    checks["untouched_rows_with_a_red_column"] = len(untouched_red)
    differing = [row.number for row in rows if (product_rows.get(row.number, {}).get("status"), product_rows.get(row.number, {}).get("device_exit"))
                 != (row.driver_fields.get("status"), row.driver_fields.get("device_exit"))]
    checks["rows_whose_directory_differs_from_the_driver_product"] = len(differing)
    lines.result(f"name=driver_product_rows rows_in_the_product={len(product_rows)} rows_in_the_root={len(rows)} "
                 f"rows_whose_status_or_exit_differs={field(','.join(map(str, differing)) or '-')}")
    return checks


def inputs_outside_this_run(dumps_roots: list[str], product_prefix: str, product_suffix: str, directories: list[str], products: list[str]) -> list[str]:
    """导出目录要在给的某一个导出根之下、产物名要带这一趟的前缀与后缀（逐个交给第一段比对器的同名函数判）。空表示都在这一趟里。"""
    outside = [path for path in directories
               if all(segment_one.inputs_outside_this_run(root, product_prefix, product_suffix, [path], []) for root in dumps_roots)]
    return outside + segment_one.inputs_outside_this_run(dumps_roots[0], product_prefix, product_suffix, [], products)


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E171 第 1 次重跑第二段的比对器")
    parser.add_argument("--baseline", required=True)
    parser.add_argument("--stability-baseline", action="append", default=[])
    parser.add_argument("--rows-root", required=True)
    parser.add_argument("--driver-product", required=True)
    parser.add_argument("--mutations-table")
    parser.add_argument("--dumps-root", action="append", required=True)
    parser.add_argument("--product-prefix", required=True)
    parser.add_argument("--product-suffix", required=True)
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    outside = inputs_outside_this_run(arguments.dumps_root, arguments.product_prefix, arguments.product_suffix,
                                      [arguments.baseline, arguments.rows_root, *arguments.stability_baseline], [arguments.driver_product])
    if outside:
        print("比对器只读这一趟的导出根与产物前缀（登记修订 8 第 8.5 节第 1 条），不在这一趟里的输入：" + "，".join(outside), file=sys.stderr)
        return 2
    lines = ResultLines(arguments.output)
    lines.result(f"name=config segment=s2 comparator=segment_two baseline={field(arguments.baseline)} "
                 f"stability_baselines={field(','.join(arguments.stability_baseline) or '-')} rows_root={field(arguments.rows_root)} "
                 f"driver_product={field(os.path.basename(arguments.driver_product))} mutations_table={field(arguments.mutations_table or '-')} "
                 f"dumps_roots={field(','.join(arguments.dumps_root))} product_prefix={field(arguments.product_prefix)} product_suffix={field(arguments.product_suffix)}")
    baseline = load_run_directory(arguments.baseline)
    unstable, stability_differing = stability_lines(lines, arguments.baseline, baseline,
                                                    [(path, load_run_directory(path)) for path in arguments.stability_baseline])
    headers, product_rows = driver_product_lines(arguments.driver_product)
    lines.result(f"name=driver_product row_time_limit_seconds={field(headers.get('row_time_limit', {}).get('seconds', '-'))} "
                 f"snapshot_digest={field(headers.get('snapshot', {}).get('snapshot_digest', '-'))} "
                 f"rows={field(headers.get('snapshot', {}).get('rows', '-'))} table_rows={field(headers.get('table', {}).get('rows', '-'))}")
    rows = [read_the_row(baseline, arguments.rows_root, number) for number in row_numbers_in(arguments.rows_root)]
    for row in rows:
        row_line(lines, row)
    q2, found = q2_lines(lines, rows, baseline, unstable)
    denominators = q3_lines(lines, rows)
    checks = check_lines(lines, rows, product_rows, arguments.mutations_table)
    row_125 = next((row for row in rows if row.number == segment_one.Q3_ROW), None)
    not_located = [row.number for row in rows if row.not_located()]
    lines.result(f"name=verdict q2={q2} q2_set_rows={len(found)} q2_o1_red_rows={sum(q2_row.comparison_red_calls > 0 for q2_row in found)} "
                 + " ".join(f"d_{key}={value}" for key, value in denominators.items())
                 + f" row_125_counted_registration_batch_1={str(row_125 is not None and row_125.category(READING_REGISTRATION, matrix.FIRST_BATCH) == CATEGORY_COUNTED).lower()}"
                 + f" rows_read={len(rows)} table_rows={TABLE_ROWS} rows_not_located={field(','.join(map(str, not_located)) or '-')}"
                 + f" unstable_memory_objects={len(unstable)} stability_baseline_files_differing={stability_differing} "
                 + " ".join(f"{key}={value}" for key, value in checks.items()))
    lines.finish()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
