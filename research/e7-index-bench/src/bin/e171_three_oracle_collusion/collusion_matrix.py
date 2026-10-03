"""E171 的矩阵：每行（一个坏实现）在每条 oracle 上红不红、触达判法、轨迹、Q1 / Q3 的谓词（登记第一节读法 3–9、第六节、第八节）。

只吃 `run_directory.py` 读回来的东西与 O3 的判定（执行器的 `name=state` 行，没跑时是 None），不碰镜像字节；
每个谓词都是纯函数，由 `test_collusion.py` 用手造数据单测（登记第九节 X1–X3、X7、X8、X11、X12）。
"""

from __future__ import annotations

from dataclasses import dataclass

from run_directory import (CRASH_JUDGE_NOT_APPLICABLE, CRASH_JUDGE_RED, POOL_CHECKER_RED, STREAM_NAMES, RunDirectory,
                           StateRow, StreamRun)

ORACLE_COMPARISON = "O1-对拍"
ORACLE_CRASH_JUDGE = "O1-崩溃判官"
ORACLE_ONE = "O1"
ORACLE_TWO = "O2"
ORACLE_THREE_STRICT = "O3甲"
ORACLE_THREE_PROVISIONAL = "O3乙"
O3_VERDICT_NOT_IN = "not_in"
O3_VERDICT_IN = "in"
O3_VERDICT_SPEC_GAP = "spec_gap"
O3_VERDICT_ERROR = "error"
FIRST_BATCH = 1
SECOND_BATCH = 2
COVERAGE_THRESHOLD_FOR_THE_STRICT_READING = 0.9
MINIMUM_COUNTED_ROWS = 30
RED_STATES_OF_A_SINGLE_POINT = 1
LAYER0_ORACLE_PREFIXES = ("record_checker:", "journal_consulted_oracle:", "journal_ignored_oracle:")
UNMOUNT_LABEL = "unmount"
ERRORED_OUTCOME_PREFIX = "err"  # `call-outcomes.tsv` 的结局列：成功 `ok`、返回错误 `err:<成员>`、panic `panic`
MODEL_NOT_ASKED_WORDS = ("not_modelled",)  # 模型因对拍判出不同停下之后的调用（`skipped_after_the_model_stopped`）是读法 6 的合规来源，不数（登记修订 4）

CATEGORY_COUNTED = "counted"
CATEGORY_UNTOUCHED = "untouched"
CATEGORY_ONLY_IN_THE_IGNORED_PASS = "only_in_the_ignored_pass"
CATEGORY_COMPILE_FAILED = "compile_failed"
CATEGORY_ANCHOR_NOT_UNIQUE = "anchor_not_unique"
CATEGORY_NOT_FOUND = "not_found"
CATEGORY_SELF_CRASH = "self_crash"
CATEGORY_TIMEOUT = "timeout"
CATEGORIES_THAT_COUNT_AGAINST_THE_TABLE = (
    CATEGORY_COMPILE_FAILED, CATEGORY_ANCHOR_NOT_UNIQUE, CATEGORY_NOT_FOUND, CATEGORY_SELF_CRASH, CATEGORY_TIMEOUT)

# O3 的判定：(流名, 状态号) → (严格, 暂代)；没跑 O3 时是 None（那几列写 not_run、不算红、不算不红）。
O3Verdicts = dict[tuple[str, int], tuple[str, str]]


@dataclass(frozen=True)
class TouchReport:
    """登记第一节读法 4 的三样与「只在不看 journal 那一遍不同」。"""

    write_stream_differs: bool
    call_outcomes_differ: bool
    recovery_report_differs: bool
    ignored_pass_differs: bool

    def touched(self) -> bool:
        return self.write_stream_differs or self.call_outcomes_differ or self.recovery_report_differs

    def only_in_the_ignored_pass(self) -> bool:
        return not self.touched() and self.ignored_pass_differs


def outcome_words(stream: StreamRun) -> list[tuple[str, str]]:
    return [(call.label, call.outcome) for call in stream.calls]


def batch_states(stream: StreamRun, batch: int) -> list[StateRow]:
    return [state for state in stream.states if state.batch == batch]


def touch_report(baseline: RunDirectory, row: RunDirectory) -> TouchReport:
    """触达：下面三样里至少一样与基线（不改实现那一趟）不同——① 两条流任一条的 ops.tsv、ops-payload.bin、calls.tsv、base-sectors.bin 的 sha256；
    ② 任一次调用的结局（成功 / 哪一种错误）；③ 写流全同时，批 1 任一态上看 journal 那一遍恢复的报告（择到的根、结局种类、读回内容的 sha256、施加的记录条数）。"""
    write_stream_differs = False
    outcomes_differ = False
    recovery_differs = False
    ignored_differs = False
    for name in STREAM_NAMES:
        base_stream, row_stream = baseline.streams[name], row.streams[name]
        if base_stream.write_stream_sha256 != row_stream.write_stream_sha256:
            write_stream_differs = True
        if outcome_words(base_stream) != outcome_words(row_stream):
            outcomes_differ = True
    if not write_stream_differs:
        for name in STREAM_NAMES:
            base_by_number = {state.state_number: state for state in batch_states(baseline.streams[name], FIRST_BATCH)}
            for state in batch_states(row.streams[name], FIRST_BATCH):
                base_state = base_by_number.get(state.state_number)
                if base_state is None or base_state.consulted != state.consulted:
                    recovery_differs = True
                if base_state is None or base_state.ignored != state.ignored:
                    ignored_differs = True
    return TouchReport(write_stream_differs, outcomes_differ, recovery_differs, ignored_differs)


def o3_verdict_of(o3: O3Verdicts | None, stream: str, state: StateRow, which: int) -> str | None:
    if o3 is None:
        return None
    verdicts = o3.get((stream, state.state_number))
    return None if verdicts is None else verdicts[which]


def state_is_red(oracle: str, stream: str, state: StateRow, o3: O3Verdicts | None) -> bool:
    """一个状态在一条 oracle 上红不红：崩溃判官 `red`；O2 `red`；O3 严格 / 暂代「不在」（欠一句、出错、弃权都不红，登记第一节读法 7）。"""
    if oracle == ORACLE_CRASH_JUDGE:
        return state.crash_judge == CRASH_JUDGE_RED
    if oracle == ORACLE_TWO:
        return state.pool_checker == POOL_CHECKER_RED
    if oracle == ORACLE_THREE_STRICT:
        return o3_verdict_of(o3, stream, state, 0) == O3_VERDICT_NOT_IN
    if oracle == ORACLE_THREE_PROVISIONAL:
        return o3_verdict_of(o3, stream, state, 1) == O3_VERDICT_NOT_IN
    raise ValueError(f"认不出的 oracle：{oracle}")


def red_calls(run: RunDirectory) -> dict[str, int]:
    """O1-对拍按调用算：每条流里模型对不上的调用数。"""
    return {name: sum(call.is_a_model_disagreement() for call in run.streams[name].calls) for name in STREAM_NAMES}


def red_state_counts(run: RunDirectory, oracle: str, batch: int, o3: O3Verdicts | None) -> dict[str, int]:
    return {name: sum(state_is_red(oracle, name, state, o3) for state in batch_states(run.streams[name], batch))
            for name in STREAM_NAMES}


def column_is_red(run: RunDirectory, oracle: str, batch: int, o3: O3Verdicts | None) -> bool | None:
    """一行（一个坏实现）在一条 oracle 上算红，当且仅当这条 oracle 在两条流的这一批里至少判红一处（O1-对拍按调用算，其余按状态算）。
    O3 两列在 O3 没跑时交回 None。O1 = 对拍或崩溃判官任一红。"""
    if oracle in (ORACLE_THREE_STRICT, ORACLE_THREE_PROVISIONAL) and o3 is None:
        return None
    if oracle == ORACLE_COMPARISON:
        return sum(red_calls(run).values()) > 0
    if oracle == ORACLE_ONE:
        return bool(column_is_red(run, ORACLE_COMPARISON, batch, o3) or column_is_red(run, ORACLE_CRASH_JUDGE, batch, o3))
    return sum(red_state_counts(run, oracle, batch, o3).values()) > 0


def crash_judge_not_applicable_counts(run: RunDirectory, batch: int) -> dict[str, int]:
    return {name: sum(state.crash_judge == CRASH_JUDGE_NOT_APPLICABLE for state in batch_states(run.streams[name], batch))
            for name in STREAM_NAMES}


def errored_calls_left_unjudged(run: RunDirectory) -> dict[str, int]:
    """O1-对拍的回查（装置逐行那一趟 S5 的比对器一侧，代码轮第三轮判决 K4-c 甲）：返回了错误、而 `call-outcomes.tsv` 里 O1-对拍一栏写的是
    没问模型（`not_modelled`）的调用数，每条流各一个数。卸载除外：模型没有卸载的答（登记第一节读法 6）；模型因对拍判出不同停下之后没比的调用
    （`skipped_after_the_model_stopped`）不数：读法 6「一次对拍判出不同之后模型不再往前走」，那一行的 O1 已经红在停下的那次调用上（登记修订 4）；
    实现 panic 的调用（结局 `panic`）走自崩那一类，不数。这个数不是 0，就是装置没把「模型说该成、实现拒了」记成对拍的红。"""
    return {name: sum(call.outcome.startswith(ERRORED_OUTCOME_PREFIX) and call.label != UNMOUNT_LABEL and call.model_word in MODEL_NOT_ASKED_WORDS
                      for call in run.streams[name].calls)
            for name in STREAM_NAMES}


def not_applicable_states_with_another_source(run: RunDirectory) -> dict[str, int]:
    """O1-崩溃判官的不适用只许来自两处（登记第一节读法 6、第十一节 S5）：模型停在卸载，或停在对拍判出不同的那次调用（装置逐行 S5 的比对器一侧，
    代码轮第三轮判决 K4-c 丙）。模型停下的那次调用不是这两种，这条流里记「不适用」的态数（两批合计）就是来源不合规的态数；模型没停、或停在这两种上，数 0。"""
    counts = {}
    for name in STREAM_NAMES:
        stream = run.streams[name]
        stopped = stream.model_stopped_at_call()
        source_is_registered = stopped is None or (
            stopped < len(stream.calls) and (stream.calls[stopped].label == UNMOUNT_LABEL or stream.calls[stopped].is_a_model_disagreement()))
        counts[name] = 0 if source_is_registered else sum(state.crash_judge == CRASH_JUDGE_NOT_APPLICABLE for state in stream.states)
    return counts


def layer0_oracle_red_in_batch(run: RunDirectory, batch: int) -> bool:
    """附带（Q1-丁）：记录核对器或层 0 手写版本表的 oracle 在这一批里判红过。"""
    return any(item.startswith(LAYER0_ORACLE_PREFIXES)
               for name in STREAM_NAMES for state in batch_states(run.streams[name], batch)
               for item in state.layer0_red_items.split(",") if item != "-")


def all_quiet(run: RunDirectory, oracles: tuple[str, ...], batch: int, o3: O3Verdicts | None) -> bool | None:
    """这几条 oracle 在这一行上都不红：任一条是 None（没跑）交回 None，任一条红交回 False，全不红 True（Q1 的「全放行」谓词）。"""
    states = [column_is_red(run, oracle, batch, o3) for oracle in oracles]
    if any(state is True for state in states):
        return False
    if any(state is None for state in states):
        return None
    return True


def o3_judged_state_count(run: RunDirectory, o3: O3Verdicts | None, which: int) -> int | None:
    """批 1 上 O3 给出「在 / 不在」的态数（不含欠一句、出错、底镜像恢复不出）。"""
    if o3 is None:
        return None
    count = 0
    for name in STREAM_NAMES:
        for state in batch_states(run.streams[name], FIRST_BATCH):
            verdict = o3_verdict_of(o3, name, state, which)
            if verdict in (O3_VERDICT_IN, O3_VERDICT_NOT_IN):
                count += 1
    return count


@dataclass(frozen=True)
class Trajectory:
    """被谓词消费的量的轨迹（登记第八节）：按段号排开的每段红态数、峰值、为正段数、期末值、第一处红在哪段哪个调用之后、单点红的段。"""

    red_by_segment: tuple[tuple[int, int], ...]
    peak: int
    positive_segments: int
    last_segment_value: int
    first_red_segment: int | None
    first_red_after_call: int | None
    single_point_segments: tuple[int, ...]

    def total(self) -> int:
        return sum(count for _, count in self.red_by_segment)


def trajectory(stream: StreamRun, stream_name: str, oracle: str, batch: int, o3: O3Verdicts | None) -> Trajectory:
    counts: dict[int, int] = {}
    first_red: StateRow | None = None
    for state in batch_states(stream, batch):
        counts.setdefault(state.segment, 0)
        if state_is_red(oracle, stream_name, state, o3):
            counts[state.segment] += 1
            if first_red is None or (state.segment, state.ordinal) < (first_red.segment, first_red.ordinal):
                first_red = state
    ordered = tuple(sorted(counts.items()))
    values = [count for _, count in ordered]
    return Trajectory(
        red_by_segment=ordered,
        peak=max(values, default=0),
        positive_segments=sum(count > 0 for count in values),
        last_segment_value=values[-1] if values else 0,
        first_red_segment=None if first_red is None else first_red.segment,
        first_red_after_call=None if first_red is None else first_red.touched_call,
        single_point_segments=single_point_segments_of(ordered),
    )


def single_point_segments_of(red_by_segment: tuple[tuple[int, int], ...],
                             red_states_that_make_a_single_point: int = RED_STATES_OF_A_SINGLE_POINT) -> tuple[int, ...]:
    """「单点红」的段（登记第八节）：这一段只红 `red_states_that_make_a_single_point` 个态，这一行在这条 oracle 上的红靠一个取样点撑着；
    判别力自证把它挪成两个态，单点红的格必须由红转不红。"""
    return tuple(segment for segment, count in red_by_segment if count == red_states_that_make_a_single_point)


def comparison_trajectory(stream: StreamRun) -> tuple[int, int | None]:
    """O1-对拍按调用的轨迹：红的调用数与第一处红在第几次调用。"""
    red = [call.index for call in stream.calls if call.is_a_model_disagreement()]
    return len(red), (red[0] if red else None)


def red_columns(run: RunDirectory, batch: int, o3: O3Verdicts | None, strict_o3: bool) -> set[str]:
    """一行红的 oracle 的集合（Q3 用）：O1、O2、O3（甲取严格、乙取暂代）。O3 没跑时 O3 不在集合里（调用方另判「没跑」）。"""
    which_o3 = ORACLE_THREE_STRICT if strict_o3 else ORACLE_THREE_PROVISIONAL
    columns = set()
    if column_is_red(run, ORACLE_ONE, batch, o3):
        columns.add(ORACLE_ONE)
    if column_is_red(run, ORACLE_TWO, batch, o3):
        columns.add(ORACLE_TWO)
    if column_is_red(run, which_o3, batch, o3):
        columns.add("O3")
    return columns


def close_one_oracle(columns: set[str], closed: str) -> set[str]:
    """关一条（登记第一节读法 9）：关掉 X = 把 X 那一列当不红，别的列照旧。"""
    return columns - {closed}


def q3_closing_report(columns: set[str]) -> dict[str, bool]:
    """逐条关一条之后这一行还有没有红：交回 {被关的 oracle: 关掉之后仍至少一条红}。红的列为空的行没有对象，交回空字典。"""
    if not columns:
        return {}
    return {closed: bool(close_one_oracle(columns, closed)) for closed in sorted({ORACLE_ONE, ORACLE_TWO, "O3"})}


def oracles_that_alone_hold_the_row(columns: set[str]) -> list[str]:
    """Q3：关掉哪一条之后这一行全放行——这一行只靠那一条撑着。红的列为空的行（本来就没被抓）没有对象。"""
    return [closed for closed, still_red in q3_closing_report(columns).items() if not still_red and closed in columns]


def coverage_enough_to_judge(counted_rows: int, minimum: int = MINIMUM_COUNTED_ROWS) -> bool:
    """Q1-分母门槛（登记第六节）：D ≥ 30 时 Q1 的「没有」够判；30 的来处是三法则（30 行全被抓时「一行全放行」的 95% 上界约 3/30）。"""
    return counted_rows >= minimum


def strict_reading_is_unreachable(zero_coverage_rows: int, counted_rows: int) -> bool:
    """F7：O3 严格跑法在 D 行里一态都没判的行数占 D 的比例 ≥ 90%（登记第十节）。D 为 0 时不触发。"""
    return counted_rows > 0 and zero_coverage_rows / counted_rows >= COVERAGE_THRESHOLD_FOR_THE_STRICT_READING
