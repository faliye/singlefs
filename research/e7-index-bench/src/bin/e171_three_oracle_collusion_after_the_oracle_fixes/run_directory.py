"""E171 第 1 次重跑装置一趟（基线、对照或坏实现一行）写出的导出目录的读回：每条流的状态表与判定、调用表、交回值、探针、流的结局，
以及写流相关文件的 sha256（原目录 `run_directory.py` 照抄起头，改成三条流与新装置的列）。

装置写的文件（第 1 次重跑登记 5.3）：E170 的十个导出文件，加 `oracle-verdicts.tsv`（25 列：原 19 列加 RefFS 的票、弃权种类或红的理由、
崩溃时刻、允许集合、恢复 panic，再加修订 5 第 5.1 节第 5 条的 `piece_panics`）、`call-outcomes.tsv`（11 列：原 9 列加交回值摘要与内存对象合摘要）、`call-return-values.tsv`（7 列）、
`writable-mount-probe.tsv`（12 列，开了探针才有）、`stream-status.tsv`。这里不解析镜像字节。
"""

from __future__ import annotations

import hashlib
import os
from dataclasses import dataclass, field

STREAM_NAMES = ("new_pool_file_creation", "fixed_script_through_e_then_unmount", "first_file_unlinked_then_inodes_across_one_leaf")
# 触达、Q2、Q3 只在流一、流二上算（登记第一节读法 2「哪个问题用哪几条流」）
TOUCH_STREAM_NAMES = STREAM_NAMES[:2]
WRITE_STREAM_FILES = ("ops.tsv", "ops-payload.bin", "calls.tsv", "base-sectors.bin")
ORACLE_VERDICT_COLUMNS = 25
CALL_OUTCOME_COLUMNS = 11
RETURN_VALUE_COLUMNS = 7
PROBE_COLUMNS = 12
STATE_COLUMNS = 6

CRASH_JUDGE_RED = "red"
CRASH_JUDGE_NOT_APPLICABLE = "na"
REFERENCE_FILESYSTEM_RED = "red"
REFERENCE_FILESYSTEM_GREEN = "green"
REFERENCE_FILESYSTEM_ABSTAINED = "abstained"
POOL_CHECKER_RED = "red"
MODEL_DISAGREES_PREFIX = "disagrees"
MODEL_JUDGED_WORDS = ("agrees",)  # 加上 disagrees:… 前缀的那些，是 O1-对拍判过的调用（读法 5「可判的调用」）


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
    reference_filesystem: str
    reference_filesystem_detail: str
    reference_filesystem_crash_moment: str
    reference_filesystem_allowed: str
    recovery_panic: str
    # 修订 5 第 5.1 节第 5 条：判态路上 panic 的件名按次序逗号连起来，没有写 `-`（只描述，不进 Q1–Q5 的谓词）
    piece_panics: str = "-"


@dataclass(frozen=True)
class CallRow:
    """`call-outcomes.tsv` 的一行：序号、名字、第一条 op、返回点、结局、O1-对拍的判定词、对不上时两边说了什么、panic 消息、交回值摘要、内存对象合摘要。"""

    index: int
    label: str
    first_operation: int
    return_point: int
    outcome: str
    model_word: str
    model_answer: str
    implementation_answer: str
    panic_message: str
    return_value_sha256: str
    memory_objects_sha256: str

    def is_a_model_disagreement(self) -> bool:
        return self.model_word.startswith(MODEL_DISAGREES_PREFIX)

    def was_judged_by_the_comparison(self) -> bool:
        return self.is_a_model_disagreement() or self.model_word in MODEL_JUDGED_WORDS

    def panicked(self) -> bool:
        return self.outcome == "panic"


@dataclass(frozen=True)
class ReturnValueRow:
    """`call-return-values.tsv` 的一行：交回值整份与顶层字段逐个的摘要、内存对象逐个的摘要、胶水交给模型的观测摘要。"""

    index: int
    label: str
    whole: str
    fields: dict[str, str]
    memory_objects: dict[str, str]
    observation: str


@dataclass(frozen=True)
class ProbeRow:
    """`writable-mount-probe.tsv` 的一行（读法 8）。"""

    batch: int
    ordinal: int
    state_number: int
    undecidable_branch: bool
    counter_branch: str
    witnessed_counter: int
    selected_version: str
    mount: str
    refusal_member: str
    refusal_on_the_d23_list: str
    second_crop: str

    def counter_branch_holds(self) -> bool:
        return self.counter_branch != "false"


@dataclass
class StreamRun:
    name: str
    exported: bool
    states: list[StateRow] = field(default_factory=list)
    calls: list[CallRow] = field(default_factory=list)
    return_values: list[ReturnValueRow] = field(default_factory=list)
    probes: list[ProbeRow] = field(default_factory=list)
    status: dict[str, str] = field(default_factory=dict)
    write_stream_sha256: dict[str, str] = field(default_factory=dict)

    def ending(self) -> str:
        return self.status.get("ending", "unknown")

    def model_stopped_at_call(self) -> int | None:
        value = self.status.get("model_stopped_at_call", "none")
        return None if value == "none" else int(value)

    def present(self) -> bool:
        """这条流在这一趟里有没有（`--streams` 没选的流目录不在）。"""
        return bool(self.status)


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
    return {columns[0]: (columns[1] if len(columns) > 1 else "") for columns in read_rows(path)}


def read_calls(path: str) -> list[CallRow]:
    calls = []
    for columns in read_rows(path):
        if len(columns) != CALL_OUTCOME_COLUMNS:
            raise ValueError(f"{path} 一行 {CALL_OUTCOME_COLUMNS} 列，读到 {len(columns)} 列：{columns}")
        calls.append(CallRow(int(columns[0]), columns[1], int(columns[2]), int(columns[3]), columns[4], columns[5],
                             columns[6], columns[7], columns[8], columns[9], columns[10]))
    return calls


def named_digests(text: str) -> dict[str, str]:
    """`名字:sha256,名字:sha256` → 字典；`-` 是空。"""
    if text == "-":
        return {}
    return dict(item.rsplit(":", 1) for item in text.split(","))


def read_return_values(path: str) -> list[ReturnValueRow]:
    rows = []
    for columns in read_rows(path):
        if len(columns) != RETURN_VALUE_COLUMNS:
            raise ValueError(f"{path} 一行 {RETURN_VALUE_COLUMNS} 列，读到 {len(columns)} 列：{columns[:3]}")
        rows.append(ReturnValueRow(int(columns[1]), columns[2], columns[3], named_digests(columns[4]), named_digests(columns[5]), columns[6]))
    return rows


def read_probes(path: str) -> list[ProbeRow]:
    rows = []
    for columns in read_rows(path):
        if len(columns) != PROBE_COLUMNS:
            raise ValueError(f"{path} 一行 {PROBE_COLUMNS} 列，读到 {len(columns)} 列：{columns}")
        rows.append(ProbeRow(int(columns[1]), int(columns[2]), int(columns[3]), columns[4] == "true", columns[5], int(columns[6]),
                             columns[7], columns[8], columns[9], columns[10], columns[11]))
    return rows


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
        states.append(StateRow(
            state_number=int(state_number), ordinal=int(ordinal), segment=int(segment), batch=int(batch),
            landing=landing, digest=digest, touched_call=None if verdict_row[4] == "-" else int(verdict_row[4]),
            crash_judge=verdict_row[5], crash_judge_detail=verdict_row[6],
            pool_checker=verdict_row[7], pool_checker_items=verdict_row[8],
            pool_checker_matches_the_judgement=verdict_row[9] == "true",
            consulted=RecoverySummary(verdict_row[10], verdict_row[11], verdict_row[12], int(verdict_row[13])),
            ignored=RecoverySummary(verdict_row[14], verdict_row[15], verdict_row[16], int(verdict_row[17])),
            layer0_red_items=verdict_row[18], reference_filesystem=verdict_row[19], reference_filesystem_detail=verdict_row[20],
            reference_filesystem_crash_moment=verdict_row[21], reference_filesystem_allowed=verdict_row[22], recovery_panic=verdict_row[23],
            piece_panics=verdict_row[24],
        ))
    return states


def load_stream_run(directory: str, name: str) -> StreamRun:
    stream_directory = os.path.join(directory, name)
    status_path = os.path.join(stream_directory, "stream-status.tsv")
    verdicts_path = os.path.join(stream_directory, "oracle-verdicts.tsv")
    if not os.path.isdir(stream_directory) or not os.path.exists(status_path):
        return StreamRun(name=name, exported=False)
    run = StreamRun(name=name, exported=os.path.exists(verdicts_path))
    run.status = read_status(status_path)
    run.calls = read_calls(os.path.join(stream_directory, "call-outcomes.tsv"))
    run.return_values = read_return_values(os.path.join(stream_directory, "call-return-values.tsv"))
    if run.exported:
        run.states = read_states(os.path.join(stream_directory, "states.tsv"), verdicts_path)
    probe_path = os.path.join(stream_directory, "writable-mount-probe.tsv")
    if os.path.exists(probe_path):
        run.probes = read_probes(probe_path)
    manifest_path = os.path.join(stream_directory, "manifest.tsv")
    manifest = read_status(manifest_path) if os.path.exists(manifest_path) else {}
    for file_name in WRITE_STREAM_FILES:
        path = os.path.join(stream_directory, file_name)
        if os.path.exists(path):
            run.write_stream_sha256[file_name] = file_sha256(path)
        elif f"sha256:{file_name}" in manifest:
            # 逐行驱动判完删掉了大文件（不带 --keep-dumps）：摘要从装置写的 manifest 取（manifest 记的是写出时那一份的 sha256）
            run.write_stream_sha256[file_name] = manifest[f"sha256:{file_name}"]
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


def result_lines_named(path: str, name: str) -> list[dict[str, str]]:
    """一份产物（或装置标准输出）里 `name=<name>` 的结果行，按原次序。"""
    found = []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            fields = parse_result_line(line.rstrip("\n"))
            if fields is not None and fields.get("name") == name:
                found.append(fields)
    return found
