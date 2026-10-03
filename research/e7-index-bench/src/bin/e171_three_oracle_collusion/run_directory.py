"""E171 装置一趟（基线或坏实现一行）写出的导出目录的读回：每条流的状态表、调用表、流的结局，以及写流相关文件的 sha256。

装置写的文件（登记 5.3）：E170 的十个导出文件，加 `oracle-verdicts.tsv`、`call-outcomes.tsv`、`stream-status.tsv`。
这里只读后三个与 `states.tsv`、`calls.tsv`、`ops.tsv` 等，不解析镜像字节。
"""

from __future__ import annotations

import hashlib
import os
from dataclasses import dataclass, field

STREAM_NAMES = ("new_pool_file_creation", "fixed_script_through_e_then_unmount")
WRITE_STREAM_FILES = ("ops.tsv", "ops-payload.bin", "calls.tsv", "base-sectors.bin")
ORACLE_VERDICT_COLUMNS = 19
CALL_OUTCOME_COLUMNS = 9
STATE_COLUMNS = 6

CRASH_JUDGE_RED = "red"
CRASH_JUDGE_GREEN = "green"
CRASH_JUDGE_NOT_APPLICABLE = "na"
CRASH_JUDGE_OFF = "off"
POOL_CHECKER_RED = "red"
POOL_CHECKER_GREEN = "green"
POOL_CHECKER_OFF = "off"
MODEL_DISAGREES_PREFIX = "disagrees"


@dataclass(frozen=True)
class RecoverySummary:
    """一遍恢复报告的摘要（装置 `RecoverySummary::text`）：择到的根、结局种类、读回内容 sha256、施加的记录条数。"""

    root: str
    outcome: str
    read_back_sha256: str
    records_applied: int


@dataclass(frozen=True)
class StateRow:
    """一个取样状态：`states.tsv` 的六列加 `oracle-verdicts.tsv` 的判定。"""

    state_number: int
    ordinal: int
    segment: int
    batch: int
    landing: str
    digest: str
    touched_call: int | None
    crash_judge: str
    crash_judge_detail: str
    pool_checker: str
    pool_checker_items: str
    pool_checker_matches_the_judgement: bool
    consulted: RecoverySummary
    ignored: RecoverySummary
    layer0_red_items: str


@dataclass(frozen=True)
class CallRow:
    """`call-outcomes.tsv` 的一行：序号、名字、第一条 op、返回点、结局、O1-对拍的判定词、对不上时两边说了什么。"""

    index: int
    label: str
    first_operation: int
    return_point: int
    outcome: str
    model_word: str
    model_answer: str
    implementation_answer: str
    panic_message: str

    def is_a_model_disagreement(self) -> bool:
        return self.model_word.startswith(MODEL_DISAGREES_PREFIX)


@dataclass
class StreamRun:
    name: str
    exported: bool
    states: list[StateRow] = field(default_factory=list)
    calls: list[CallRow] = field(default_factory=list)
    status: dict[str, str] = field(default_factory=dict)
    write_stream_sha256: dict[str, str] = field(default_factory=dict)

    def ending(self) -> str:
        return self.status.get("ending", "unknown")

    def model_stopped_at_call(self) -> int | None:
        value = self.status.get("model_stopped_at_call", "none")
        return None if value == "none" else int(value)

    def implementation_crashed(self) -> bool:
        return self.ending().startswith("implementation_crashed")


@dataclass
class RunDirectory:
    path: str
    streams: dict[str, StreamRun]


def read_rows(path: str) -> list[list[str]]:
    with open(path, encoding="utf-8") as handle:
        return [line.rstrip("\n").split("\t") for line in handle if line.strip()]


def file_sha256(path: str) -> str:
    hasher = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            hasher.update(block)
    return hasher.hexdigest()


def read_status(path: str) -> dict[str, str]:
    status = {}
    for columns in read_rows(path):
        status[columns[0]] = columns[1] if len(columns) > 1 else ""
    return status


def read_calls(path: str) -> list[CallRow]:
    calls = []
    for columns in read_rows(path):
        if len(columns) != CALL_OUTCOME_COLUMNS:
            raise ValueError(f"{path} 一行 {CALL_OUTCOME_COLUMNS} 列，读到 {len(columns)} 列：{columns}")
        calls.append(CallRow(int(columns[0]), columns[1], int(columns[2]), int(columns[3]), columns[4], columns[5],
                             columns[6], columns[7], columns[8]))
    return calls


def read_states(states_path: str, verdicts_path: str) -> list[StateRow]:
    state_columns = read_rows(states_path)
    verdict_columns = read_rows(verdicts_path)
    if len(state_columns) != len(verdict_columns):
        raise ValueError(f"{states_path} 与 {verdicts_path} 行数不同：{len(state_columns)} 对 {len(verdict_columns)}")
    states = []
    for states_row, verdict_row in zip(state_columns, verdict_columns):
        if len(states_row) != STATE_COLUMNS or len(verdict_row) != ORACLE_VERDICT_COLUMNS:
            raise ValueError(f"列数不对：{states_row} / {verdict_row}")
        state_number, ordinal, segment, batch, landing, digest = states_row
        if (verdict_row[0], verdict_row[1], verdict_row[2], verdict_row[3]) != (state_number, batch, ordinal, segment):
            raise ValueError(f"状态号、批号、序号、段号两个文件对不上：{states_row} / {verdict_row[:4]}")
        touched = None if verdict_row[4] == "-" else int(verdict_row[4])
        states.append(StateRow(
            state_number=int(state_number), ordinal=int(ordinal), segment=int(segment), batch=int(batch),
            landing=landing, digest=digest, touched_call=touched,
            crash_judge=verdict_row[5], crash_judge_detail=verdict_row[6],
            pool_checker=verdict_row[7], pool_checker_items=verdict_row[8],
            pool_checker_matches_the_judgement=verdict_row[9] == "true",
            consulted=RecoverySummary(verdict_row[10], verdict_row[11], verdict_row[12], int(verdict_row[13])),
            ignored=RecoverySummary(verdict_row[14], verdict_row[15], verdict_row[16], int(verdict_row[17])),
            layer0_red_items=verdict_row[18],
        ))
    return states


def load_stream_run(directory: str, name: str) -> StreamRun:
    stream_directory = os.path.join(directory, name)
    status_path = os.path.join(stream_directory, "stream-status.tsv")
    calls_path = os.path.join(stream_directory, "call-outcomes.tsv")
    verdicts_path = os.path.join(stream_directory, "oracle-verdicts.tsv")
    if not os.path.isdir(stream_directory) or not os.path.exists(status_path):
        return StreamRun(name=name, exported=False)
    run = StreamRun(name=name, exported=os.path.exists(verdicts_path))
    run.status = read_status(status_path)
    run.calls = read_calls(calls_path)
    if run.exported:
        run.states = read_states(os.path.join(stream_directory, "states.tsv"), verdicts_path)
    for file_name in WRITE_STREAM_FILES:
        path = os.path.join(stream_directory, file_name)
        if os.path.exists(path):
            run.write_stream_sha256[file_name] = file_sha256(path)
    return run


def load_run_directory(directory: str) -> RunDirectory:
    return RunDirectory(directory, {name: load_stream_run(directory, name) for name in STREAM_NAMES})


def parse_result_line(line: str) -> dict[str, str] | None:
    """`E7RESULT key=value key=value …` 一行拆成字典；不是结果行返回 None。"""
    if not line.startswith("E7RESULT "):
        return None
    fields = {}
    for token in line[len("E7RESULT "):].split(" "):
        key, separator, value = token.partition("=")
        if separator:
            fields[key] = value
    return fields


def device_stream_summaries(device_output_path: str) -> dict[str, dict[str, str]]:
    """装置标准输出里每条流的 `name=stream` 行：驱动怎么结束的、有没有导出、没导出的原因、状态数。"""
    summaries: dict[str, dict[str, str]] = {}
    with open(device_output_path, encoding="utf-8") as handle:
        for line in handle:
            fields = parse_result_line(line.rstrip("\n"))
            if fields is not None and fields.get("name") == "stream":
                summaries[fields["stream"]] = fields
    return summaries


def device_verdict_line(device_output_path: str) -> dict[str, str]:
    """装置标准输出里的 `name=verdict` 行（判决行）。"""
    with open(device_output_path, encoding="utf-8") as handle:
        for line in handle:
            fields = parse_result_line(line.rstrip("\n"))
            if fields is not None and fields.get("name") == "verdict":
                return fields
    return {}
