"""E171 基线那一趟的核对（登记第七节、第十一节）：Z1 两次基线逐字节相同、S2 与 E170 导出装置的同一棵树上写出的文件逐字节相同、
A3 基线上每条 oracle 判红 0 处，以及 S5 末尾那两个「装置与比对器各算一遍、互比」的数（批 1 每条流的态数、O1-崩溃判官适用的态数）。

比对器这一侧的两个数不读装置的计数：批 1 的态数从 `ops.tsv`（切段）与 `torn.tsv`（哪几次写取三态）按 E170 登记 5.4 的取样规则现算，
O1-崩溃判官适用的态数从 `calls.tsv`、`ops.tsv` 与 `states.tsv` 的落地向量现算（写落在 `unmount` 那次调用里的态不适用）。
"""

from __future__ import annotations

import os

from run_directory import STREAM_NAMES, file_sha256, read_rows

IMPLEMENTATION_VERDICTS_FILE = "impl-verdicts.tsv"
MANIFEST_FILE = "manifest.tsv"
NEUTRAL_FILES_COMPARED_WITH_E170 = ("base-sectors.bin", "ops.tsv", "ops-payload.bin", "calls.tsv", "torn.tsv",
                                    "torn-payload.bin", "states.tsv", "quick-tier.tsv")
FILES_COMPARED_BETWEEN_TWO_BASELINES = NEUTRAL_FILES_COMPARED_WITH_E170 + (
    IMPLEMENTATION_VERDICTS_FILE, MANIFEST_FILE, "oracle-verdicts.tsv", "call-outcomes.tsv", "stream-status.tsv")
FIRST_BATCH_DIVISOR = 7
TORN_STATES_PER_SEGMENT = 4
TORN_SCAN_ORDINALS_PER_SEGMENT = 100_000
UNMOUNT_LABEL = "unmount"
BARRIER = "barrier"
WRITE_FUA = "write_fua"


def contents_without_timing(file_name: str, contents: bytes) -> bytes:
    """`impl-verdicts.tsv` 每行去掉最后一列（耗时）；`manifest.tsv` 去掉记 `impl-verdicts.tsv` 摘要的那一行；其余原样（E170 登记第十二节 V2b）。"""
    if file_name == IMPLEMENTATION_VERDICTS_FILE:
        lines = contents.decode("utf-8").split("\n")
        kept = [line.rsplit("\t", 1)[0] if "\t" in line else line for line in lines]
        return "\n".join(kept).encode("utf-8")
    if file_name == MANIFEST_FILE:
        lines = contents.decode("utf-8").split("\n")
        return "\n".join(line for line in lines if not line.startswith(f"sha256:{IMPLEMENTATION_VERDICTS_FILE}\t")).encode("utf-8")
    return contents


def read_without_timing(path: str) -> bytes:
    with open(path, "rb") as handle:
        return contents_without_timing(os.path.basename(path), handle.read())


def files_identical(first_directory: str, second_directory: str, file_name: str) -> bool:
    first, second = os.path.join(first_directory, file_name), os.path.join(second_directory, file_name)
    if not (os.path.exists(first) and os.path.exists(second)):
        return False
    return read_without_timing(first) == read_without_timing(second)


def compare_two_baselines(first: str, second: str) -> list[tuple[str, str, bool]]:
    """Z1：基线连跑两次，导出目录里的文件逐个比（`impl-verdicts.tsv` 与 `manifest.tsv` 去掉耗时那一份）。交回 (流名, 文件名, 逐字节相同)。"""
    return [(stream, file_name, files_identical(os.path.join(first, stream), os.path.join(second, stream), file_name))
            for stream in STREAM_NAMES for file_name in FILES_COMPARED_BETWEEN_TWO_BASELINES]


def compare_with_e170_dump(baseline: str, e170_dump: str) -> list[tuple[str, str, bool, str]]:
    """S2：同一棵树上 E170 导出装置写的八个文件与本装置基线逐字节相同，`impl-verdicts.tsv` 去掉耗时列之后也相同。
    交回 (流名, 文件名, 相同, 本装置那份的 sha256 前 16 位)。"""
    results = []
    for stream in STREAM_NAMES:
        for file_name in NEUTRAL_FILES_COMPARED_WITH_E170 + (IMPLEMENTATION_VERDICTS_FILE,):
            ours = os.path.join(baseline, stream, file_name)
            theirs = os.path.join(e170_dump, stream, file_name)
            identical = files_identical(os.path.join(baseline, stream), os.path.join(e170_dump, stream), file_name)
            results.append((stream, file_name, identical, file_sha256(ours)[:16] if os.path.exists(ours) else "-"))
            _ = theirs
    return results


def baseline_red_counts(baseline: str) -> dict[str, int]:
    """A3：基线两条流所有批的状态上，O1-崩溃判官、O2 判红的态数与模型对不上的调用数（都要是 0）。"""
    counts = {"o1_comparison_red_calls": 0, "o1_crash_judge_red_states": 0, "o2_red_states": 0}
    for stream in STREAM_NAMES:
        for columns in read_rows(os.path.join(baseline, stream, "call-outcomes.tsv")):
            counts["o1_comparison_red_calls"] += columns[5].startswith("disagrees")
        for columns in read_rows(os.path.join(baseline, stream, "oracle-verdicts.tsv")):
            counts["o1_crash_judge_red_states"] += columns[5] == "red"
            counts["o2_red_states"] += columns[7] == "red"
    return counts


# ───────────────────────────── S5：比对器自己现算的两个数 ─────────────────────────────

def segments_of_the_write_stream(operation_rows: list[list[str]]) -> list[list[int]]:
    """切段（`singlefs_harness::segments::SegmentClosingRule` 的规则，比对器自己写一遍）：普通写与清零记进当前段、把它的盘记成「没放行」；
    FUA 写记进当前段、放行它自己那块盘；屏障放行它那块盘；当前段里有写、而每块盘都放行了，这一段关上。流尾没关的段单独成一段。
    交回每段里的写序号（写序号按非屏障操作的次序数）。"""
    segments: list[list[int]] = []
    current: list[int] = []
    unreleased: set[str] = set()
    write_index = 0
    for columns in operation_rows:
        device, kind = columns[1], columns[2]
        if kind == BARRIER:
            unreleased.discard(device)
        else:
            current.append(write_index)
            write_index += 1
            if kind == WRITE_FUA:
                unreleased.discard(device)
            else:
                unreleased.add(device)
        if current and not unreleased and kind in (BARRIER, WRITE_FUA):
            segments.append(current)
            current = []
    if current:
        segments.append(current)
    return segments


def state_ranges_of_the_segments(segments: list[list[int]], tearable_writes: set[int]) -> tuple[list[range], int]:
    """层 0 全量展开（`EveryProperSubset`）的每段序号区间（首尾相接）与状态总数：一段的状态数是它每次写取几态（原地覆写取三态、别的两态）
    的乘积减 1（真子集：不含「这一段的写全持久」那一个，它是下一段的起点），总数是各段之和再加最后全部持久那一个。"""
    ranges = []
    next_ordinal = 0
    for segment in segments:
        count = 1
        for write_index in segment:
            count *= 3 if write_index in tearable_writes else 2
        count -= 1
        ranges.append(range(next_ordinal, next_ordinal + count))
        next_ordinal += count
    return ranges, next_ordinal + 1


def landing_vector_of_ordinal(ordinal: int, segments: list[list[int]], ranges: list[range], tearable_writes: set[int],
                              write_count: int) -> str:
    """序号 → 落地向量：之前的段整段持久（`2`）；所在段的序号按混合进制拆到各次写上（第一次写是最低位；两态 0 没持久 1 持久、三态 0 没持久 1 撕裂 2 持久）；之后没持久。"""
    landings = ["0"] * write_count
    segment_index = next((index for index, state_range in enumerate(ranges) if ordinal in state_range), len(ranges))
    for segment in segments[:segment_index]:
        for write_index in segment:
            landings[write_index] = "2"
    if segment_index < len(ranges):
        remaining = ordinal - ranges[segment_index].start
        for write_index in segments[segment_index]:
            choices = 3 if write_index in tearable_writes else 2
            digit = remaining % choices
            remaining //= choices
            landings[write_index] = "0" if digit == 0 else ("2" if choices == 2 or digit == 2 else "1")
    elif segments:
        landings = ["2"] * write_count
    return "".join(landings)


def first_batch_ordinals(segments: list[list[int]], ranges: list[range], total: int, tearable_writes: set[int],
                         write_count: int) -> set[int]:
    """批 1（E170 登记 5.4）：每个非空区间 [a, b) 取 a、b − 1 与 a + ⌊i·(b − 1 − a)/7⌋（i = 1..6）；再加 T − 1；再对有撕裂写的每一段，
    在区间里按序号升序扫（每段最多扫 100 000 个序号），取落地向量里带 `1` 的前 4 个、不与已取的重复。"""
    chosen: set[int] = set()
    for state_range in ranges:
        if len(state_range) == 0:
            continue
        first, span = state_range.start, state_range.stop - 1 - state_range.start
        chosen.add(first)
        chosen.add(state_range.stop - 1)
        for step in range(1, FIRST_BATCH_DIVISOR):
            chosen.add(first + step * span // FIRST_BATCH_DIVISOR)
    chosen.add(total - 1)
    for segment, state_range in zip(segments, ranges):
        if len(state_range) == 0 or not any(write_index in tearable_writes for write_index in segment):
            continue
        found = 0
        for ordinal in range(state_range.start, min(state_range.stop, state_range.start + TORN_SCAN_ORDINALS_PER_SEGMENT)):
            if found == TORN_STATES_PER_SEGMENT:
                break
            if "1" in landing_vector_of_ordinal(ordinal, segments, ranges, tearable_writes, write_count) and ordinal not in chosen:
                chosen.add(ordinal)
                found += 1
    return chosen


def recompute_first_batch(stream_directory: str) -> set[int]:
    operations = read_rows(os.path.join(stream_directory, "ops.tsv"))
    tearable = {int(columns[0]) for columns in read_rows(os.path.join(stream_directory, "torn.tsv"))}
    write_count = sum(1 for columns in operations if columns[2] != BARRIER)
    segments = segments_of_the_write_stream(operations)
    ranges, total = state_ranges_of_the_segments(segments, tearable)
    return first_batch_ordinals(segments, ranges, total, tearable, write_count)


def recompute_crash_judge_applicable(stream_directory: str, batch: int) -> int:
    """基线上 O1-崩溃判官适用的态数：落地向量里有 `unmount` 那次调用的写持久或撕裂的态不适用，别的态都适用。流一没有 `unmount`。"""
    calls = read_rows(os.path.join(stream_directory, "calls.tsv"))
    operations = read_rows(os.path.join(stream_directory, "ops.tsv"))
    unmount_calls = [int(columns[0]) for columns in calls if columns[1] == UNMOUNT_LABEL]
    write_operation_indices = [index for index, columns in enumerate(operations) if columns[2] != BARRIER]
    first_operations = [int(columns[2]) for columns in calls]
    applicable = 0
    for columns in read_rows(os.path.join(stream_directory, "states.tsv")):
        if int(columns[3]) != batch:
            continue
        touched = [
            max((call_index for call_index, first in enumerate(first_operations) if first <= write_operation_indices[write_index]),
                default=None)
            for write_index, landing in enumerate(columns[4]) if landing != "0"]
        touches_unmount = any(call in unmount_calls for call in touched if call is not None)
        applicable += not touches_unmount
    return applicable


# ───────────────────────────── Q3-核：关一条之后其余列与不关时逐字节相同（登记第一节读法 9、第六节 Q3-核） ─────────────────────────────

ORACLE_VERDICT_COLUMNS_OF_O1 = (5, 6)
ORACLE_VERDICT_COLUMNS_OF_O2 = (7, 8, 9)
CALL_OUTCOME_COLUMNS_OF_O1 = (5, 6, 7)
STREAM_STATUS_FILE = "stream-status.tsv"
STREAM_STATUS_KEYS_OF_O1 = ("model_stopped_at_call",)  # 模型停在第几次调用：O1 自己的状态（关掉 O1 模型就不停），登记修订 4


def rows_without_columns(path: str, dropped: tuple[int, ...]) -> list[list[str]]:
    return [[value for index, value in enumerate(columns) if index not in dropped] for columns in read_rows(path)]


def rows_without_keys(path: str, dropped_keys: tuple[str, ...]) -> list[list[str]]:
    """`stream-status.tsv` 这种 key<TAB>value 的文件去掉被关那条 oracle 自己的几行；别的行原样留着照比。"""
    return [columns for columns in read_rows(path) if not (columns and columns[0] in dropped_keys)]


def differences_after_closing_one_oracle(full_directory: str, closed_directory: str, closed: str) -> list[str]:
    """装置 `--disable-oracle O1|O2` 的一趟与不关的一趟比：除了被关那一列自己，别的文件与别的列要逐字节相同。交回不同的地方（空表示都相同）。
    O1 的列是 `oracle-verdicts.tsv` 的 5–6 列、`call-outcomes.tsv` 的 5–7 列与 `stream-status.tsv` 的 `model_stopped_at_call` 那一行；
    O2 的列是 `oracle-verdicts.tsv` 的 7–9 列（关 O2 时 `stream-status.tsv` 整份照比）。"""
    if closed not in ("O1", "O2"):
        raise ValueError(f"装置只认 O1、O2：{closed}")
    differences = []
    dropped_verdicts = ORACLE_VERDICT_COLUMNS_OF_O1 if closed == "O1" else ORACLE_VERDICT_COLUMNS_OF_O2
    dropped_calls = CALL_OUTCOME_COLUMNS_OF_O1 if closed == "O1" else ()
    dropped_status_keys = STREAM_STATUS_KEYS_OF_O1 if closed == "O1" else ()
    for stream in STREAM_NAMES:
        full_stream, closed_stream = os.path.join(full_directory, stream), os.path.join(closed_directory, stream)
        for file_name in sorted(os.listdir(full_stream)):
            if file_name in ("oracle-verdicts.tsv", "call-outcomes.tsv", STREAM_STATUS_FILE):
                continue
            if not files_identical(full_stream, closed_stream, file_name):
                differences.append(f"{stream}/{file_name}")
        if rows_without_columns(os.path.join(full_stream, "oracle-verdicts.tsv"), dropped_verdicts) != \
                rows_without_columns(os.path.join(closed_stream, "oracle-verdicts.tsv"), dropped_verdicts):
            differences.append(f"{stream}/oracle-verdicts.tsv（被关的列之外）")
        if rows_without_columns(os.path.join(full_stream, "call-outcomes.tsv"), dropped_calls) != \
                rows_without_columns(os.path.join(closed_stream, "call-outcomes.tsv"), dropped_calls):
            differences.append(f"{stream}/call-outcomes.tsv（被关的列之外）")
        full_status, closed_status = os.path.join(full_stream, STREAM_STATUS_FILE), os.path.join(closed_stream, STREAM_STATUS_FILE)
        if not (os.path.exists(full_status) and os.path.exists(closed_status)) or \
                rows_without_keys(full_status, dropped_status_keys) != rows_without_keys(closed_status, dropped_status_keys):
            differences.append(f"{stream}/{STREAM_STATUS_FILE}（被关的那一行之外）")
    return differences


def differences_when_nothing_is_closed(full_directory: str, other_directory: str) -> list[str]:
    """Q3-核「驱动不跑执行器」那一次：装置照原样跑（没关任何一条），导出目录里每个文件都要与不关的那一趟逐字节相同（耗时去掉）。交回不同的文件。"""
    differences = []
    for stream in STREAM_NAMES:
        full_stream, other_stream = os.path.join(full_directory, stream), os.path.join(other_directory, stream)
        names = sorted(set(os.listdir(full_stream)) | set(os.listdir(other_stream)))
        differences.extend(f"{stream}/{name}" for name in names if not files_identical(full_stream, other_stream, name))
    return differences


def o3_state_lines_without_timing(path: str) -> list[str]:
    """执行器产物里每态一行 `name=state …` 去掉每态耗时（`time_ns`）与工作进程号（`worker`）、按（流名、状态号）排好；
    多进程时行的次序不定，所以排序之后才能逐行比。Q3-核「其余列逐字节相同」里 O3 那一列用它比。"""
    rows = []
    with open(path, encoding="utf-8") as handle:
        for raw in handle:
            line = raw.rstrip("\n").removeprefix("E7RESULT ")
            if not line.startswith("name=state "):
                continue
            fields = dict(token.split("=", 1) for token in line.split(" ") if "=" in token)
            kept = " ".join(token for token in line.split(" ") if not token.startswith(("time_ns=", "worker=")))
            rows.append(((fields["stream"], int(fields["state"])), kept))
    return [text for _, text in sorted(rows)]
