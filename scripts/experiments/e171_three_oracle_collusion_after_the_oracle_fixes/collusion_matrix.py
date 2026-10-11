"""E171 第 1 次重跑的矩阵：一行（一个坏实现）在每条 oracle、每个子列上红不红，触达，Q2 的交回值 / 内存对象差，Q3 的可判调用数，Q4 的探针计数，
轨迹（第 1 次重跑登记第一节读法 1–8、第六节、第八节）。由原目录 `collusion_matrix.py` 照抄起头，改成 O1 三个子列、三条流。

只吃 `run_directory.py` 读回来的东西与 O3 的判定（执行器的 `name=state` 行，没跑时是 None），不碰镜像字节；每个谓词都是纯函数，
由 `test_segment_one.py` 用手造数据单测（登记第九节 V13–V16 与第八节判别力自证）。
"""

from __future__ import annotations

from dataclasses import dataclass

from run_directory import (CRASH_JUDGE_NOT_APPLICABLE, CRASH_JUDGE_RED, POOL_CHECKER_RED, REFERENCE_FILESYSTEM_RED, STREAM_NAMES,
                           TOUCH_STREAM_NAMES, ProbeRow, RunDirectory, StateRow, StreamRun)

SUBCOLUMN_COMPARISON = "O1-对拍"
SUBCOLUMN_CRASH_JUDGE = "O1-模型崩溃判官"
SUBCOLUMN_REFERENCE_FILESYSTEM = "O1-RefFS"
O1_SUBCOLUMNS = (SUBCOLUMN_COMPARISON, SUBCOLUMN_CRASH_JUDGE, SUBCOLUMN_REFERENCE_FILESYSTEM)
ORACLE_ONE = "O1"
ORACLE_TWO = "O2"
ORACLE_THREE_STRICT = "O3甲"
ORACLE_THREE_PROVISIONAL = "O3乙"
O3_VERDICT_NOT_IN = "not_in"
O3_VERDICT_IN = "in"
FIRST_BATCH = 1
SECOND_BATCH = 2
RED_STATES_OF_A_SINGLE_POINT = 1
ERRORED_OUTCOME_PREFIX = "err"  # `call-outcomes.tsv` 的结局列：成功 `ok`、返回错误 `err:<成员>`、panic `panic`、在途 `in_flight`
MODEL_NOT_ASKED_WORD = "not_modelled"
MINIMUM_JUDGED_CALLS_BEFORE_THE_FIRST_PANIC = 1  # 读法 5：流一、流二合计 0 次就是「实现自崩」（问题单 Q3 翻面观测「自崩前可判的调用数为 0」）

# O3 的判定：(流名, 状态号) → (严格, 暂代)；没跑 O3 时是 None（那几列写 not_run、不算红、不算不红）。
O3Verdicts = dict[tuple[str, int], tuple[str, str]]


@dataclass(frozen=True)
class TouchReport:
    """登记第一节读法 4 的三样（只在流一、流二上算）与「只在不看 journal 那一遍不同」。"""

    write_stream_differs: bool
    call_outcomes_differ: bool
    recovery_report_differs: bool
    ignored_pass_differs: bool

    def touched(self) -> bool:
        return self.write_stream_differs or self.call_outcomes_differ or self.recovery_report_differs


def outcome_words(stream: StreamRun) -> list[tuple[str, str]]:
    return [(call.label, call.outcome) for call in stream.calls]


def batch_states(stream: StreamRun, batch: int) -> list[StateRow]:
    return [state for state in stream.states if state.batch == batch]


def touch_report(baseline: RunDirectory, row: RunDirectory) -> TouchReport:
    """触达（原登记读法 4 一个字不改，只在流一、流二上算）：① 两条流任一条的 ops.tsv、ops-payload.bin、calls.tsv、base-sectors.bin 的 sha256；
    ② 任一次调用的结局；③ 写流全同时，批 1 任一态上看 journal 那一遍恢复的报告。"""
    write_stream_differs = outcomes_differ = recovery_differs = ignored_differs = False
    for name in TOUCH_STREAM_NAMES:
        base_stream, row_stream = baseline.streams[name], row.streams[name]
        write_stream_differs |= base_stream.write_stream_sha256 != row_stream.write_stream_sha256
        outcomes_differ |= outcome_words(base_stream) != outcome_words(row_stream)
    if not write_stream_differs:
        for name in TOUCH_STREAM_NAMES:
            base_by_number = {state.state_number: state for state in batch_states(baseline.streams[name], FIRST_BATCH)}
            for state in batch_states(row.streams[name], FIRST_BATCH):
                base_state = base_by_number.get(state.state_number)
                recovery_differs |= base_state is None or base_state.consulted != state.consulted
                ignored_differs |= base_state is None or base_state.ignored != state.ignored
    return TouchReport(write_stream_differs, outcomes_differ, recovery_differs, ignored_differs)


def o3_verdict_of(o3: O3Verdicts | None, stream: str, state: StateRow, which: int) -> str | None:
    if o3 is None:
        return None
    verdicts = o3.get((stream, state.state_number))
    return None if verdicts is None else verdicts[which]


def state_is_red(subcolumn_or_oracle: str, stream: str, state: StateRow, o3: O3Verdicts | None) -> bool:
    """一个状态在一个子列或一条 oracle 上红不红：模型崩溃判官 `red`；RefFS `red`（弃权不算红，读法 1）；O2 `red`；
    O3 严格 / 暂代「不在」（欠一句、出错、弃权都不红，原登记读法 7）。"""
    if subcolumn_or_oracle == SUBCOLUMN_CRASH_JUDGE:
        return state.crash_judge == CRASH_JUDGE_RED
    if subcolumn_or_oracle == SUBCOLUMN_REFERENCE_FILESYSTEM:
        return state.reference_filesystem == REFERENCE_FILESYSTEM_RED
    if subcolumn_or_oracle == ORACLE_TWO:
        return state.pool_checker == POOL_CHECKER_RED
    if subcolumn_or_oracle == ORACLE_THREE_STRICT:
        return o3_verdict_of(o3, stream, state, 0) == O3_VERDICT_NOT_IN
    if subcolumn_or_oracle == ORACLE_THREE_PROVISIONAL:
        return o3_verdict_of(o3, stream, state, 1) == O3_VERDICT_NOT_IN
    raise ValueError(f"认不出的子列或 oracle：{subcolumn_or_oracle}")


def streams_present(run: RunDirectory, streams: tuple[str, ...] = STREAM_NAMES) -> list[str]:
    return [name for name in streams if run.streams[name].present()]


def red_calls(run: RunDirectory, streams: tuple[str, ...] = STREAM_NAMES) -> dict[str, int]:
    """O1-对拍按调用算：每条流里模型对不上的调用数。"""
    return {name: sum(call.is_a_model_disagreement() for call in run.streams[name].calls) for name in streams_present(run, streams)}


def red_state_counts(run: RunDirectory, subcolumn_or_oracle: str, batch: int, o3: O3Verdicts | None,
                     streams: tuple[str, ...] = STREAM_NAMES) -> dict[str, int]:
    return {name: sum(state_is_red(subcolumn_or_oracle, name, state, o3) for state in batch_states(run.streams[name], batch))
            for name in streams_present(run, streams)}


def column_is_red(run: RunDirectory, oracle: str, batch: int, o3: O3Verdicts | None,
                  streams: tuple[str, ...] = STREAM_NAMES) -> bool | None:
    """一行在一条 oracle（或 O1 的一个子列）上算红，当且仅当它在这几条流的这一批里至少判红一处（O1-对拍按调用算，其余按状态算，读法 3）。
    O1 = 三个子列任一个红（读法 1）。O3 两列在 O3 没跑时交回 None。"""
    if oracle in (ORACLE_THREE_STRICT, ORACLE_THREE_PROVISIONAL) and o3 is None:
        return None
    if oracle == SUBCOLUMN_COMPARISON:
        return sum(red_calls(run, streams).values()) > 0
    if oracle == ORACLE_ONE:
        return any(column_is_red(run, subcolumn, batch, o3, streams) for subcolumn in O1_SUBCOLUMNS)
    return sum(red_state_counts(run, oracle, batch, o3, streams).values()) > 0


def all_quiet(run: RunDirectory, oracles: tuple[str, ...], batch: int, o3: O3Verdicts | None,
              streams: tuple[str, ...] = STREAM_NAMES) -> bool | None:
    """这几条 oracle 在这一行上都不红：任一条红交回 False，任一条没跑交回 None，全不红 True（Q1 的「三条全放行」谓词）。"""
    states = [column_is_red(run, oracle, batch, o3, streams) for oracle in oracles]
    if any(state is True for state in states):
        return False
    if any(state is None for state in states):
        return None
    return True


def errored_calls_left_unjudged(run: RunDirectory) -> dict[str, int]:
    """S5 的比对器一侧（读法 6）：交回了错误（`err:` 起头）、而 O1-对拍一栏是 `not_modelled` 的调用数。卸载由模型答之后没有例外；
    panic（`panic`）与模型停下之后没比的调用不数。"""
    return {name: sum(call.outcome.startswith(ERRORED_OUTCOME_PREFIX) and call.model_word == MODEL_NOT_ASKED_WORD
                      for call in run.streams[name].calls)
            for name in streams_present(run)}


def not_applicable_states_with_another_source(run: RunDirectory) -> dict[str, int]:
    """O1-模型崩溃判官的不适用只许来自三处（读法 6）：模型停在对拍判出不同、交回错误、或 panic 的那次调用。模型停下的那次调用不是这三种，
    这条流里记「不适用」的态数（两批合计）就是来源不合规的态数（停机 S5）。"""
    counts = {}
    for name in streams_present(run):
        stream = run.streams[name]
        stopped = stream.model_stopped_at_call()
        registered = stopped is None or (stopped < len(stream.calls) and (
            stream.calls[stopped].is_a_model_disagreement() or stream.calls[stopped].outcome.startswith(ERRORED_OUTCOME_PREFIX)
            or stream.calls[stopped].panicked()))
        counts[name] = 0 if registered else sum(state.crash_judge == CRASH_JUDGE_NOT_APPLICABLE for state in stream.states)
    return counts


def judged_calls_before_the_first_panic(run: RunDirectory) -> int | None:
    """Q3（读法 5）：流一、流二上第一处调用 panic 之前 O1-对拍判过（`agrees` 或 `disagrees`）的调用数，两流合计；两条流都没有 panic 交回 None。"""
    saw_a_panic = False
    judged = 0
    for name in TOUCH_STREAM_NAMES:
        calls = run.streams[name].calls
        panic_at = next((call.index for call in calls if call.panicked()), None)
        if panic_at is None:
            continue
        saw_a_panic = True
        judged += sum(call.was_judged_by_the_comparison() for call in calls[:panic_at])
    return judged if saw_a_panic else None


def counts_as_a_self_crash(run: RunDirectory, minimum: int = MINIMUM_JUDGED_CALLS_BEFORE_THE_FIRST_PANIC) -> bool:
    """读法 5「不进分母」收窄之后的实现自崩：流一、流二上有调用 panic，而 panic 之前 O1-对拍判过的调用两流合计少于 `minimum`（登记写 0 次，即少于 1）。"""
    judged = judged_calls_before_the_first_panic(run)
    return judged is not None and judged < minimum


@dataclass(frozen=True)
class ReturnValueDifference:
    """Q2：一次调用的交回值、内存对象与基线同一条流、同一次调用比出的不同（读法 4）。"""

    stream: str
    call_index: int
    differing_fields: tuple[str, ...]
    differing_objects: tuple[str, ...]
    observation_differs: bool

    def any_difference(self) -> bool:
        return bool(self.differing_fields or self.differing_objects)


def return_value_differences(baseline: RunDirectory, row: RunDirectory, unstable_objects: frozenset[str] = frozenset(),
                             streams: tuple[str, ...] = TOUCH_STREAM_NAMES) -> list[ReturnValueDifference]:
    """每条流、每次调用：交回值顶层字段哪几个与基线不同（交回值整份不同而字段名对不上的记 `#whole`）、内存对象哪几个不同（基线两次就不稳的
    `unstable_objects` 拿掉），胶水交给模型的观测同不同。基线没有这次调用的（行多走了几步）整份记不同。"""
    differences = []
    for name in streams:
        base_by_index = {value.index: value for value in baseline.streams[name].return_values}
        for value in row.streams[name].return_values:
            base = base_by_index.get(value.index)
            if base is None:
                differences.append(ReturnValueDifference(name, value.index, ("#no_baseline_call",), ("#no_baseline_call",), True))
                continue
            fields = tuple(sorted(key for key in set(base.fields) | set(value.fields) if base.fields.get(key) != value.fields.get(key)))
            if base.whole != value.whole and not fields:
                fields = ("#whole",)
            objects = tuple(sorted(key for key in set(base.memory_objects) | set(value.memory_objects)
                                   if key not in unstable_objects and base.memory_objects.get(key) != value.memory_objects.get(key)))
            difference = ReturnValueDifference(name, value.index, fields, objects, base.observation != value.observation)
            if difference.any_difference():
                differences.append(difference)
    return differences


def unstable_memory_objects(first_baseline: RunDirectory, second_baseline: RunDirectory) -> frozenset[str]:
    """基线连跑两次摘要不同的内存对象名（读法 4：从比较里拿掉、单列名字）。交回值整份不同的也记成 `#return_value`。"""
    unstable = set()
    for name in STREAM_NAMES:
        second_by_index = {value.index: value for value in second_baseline.streams[name].return_values}
        for value in first_baseline.streams[name].return_values:
            other = second_by_index.get(value.index)
            if other is None:
                continue
            unstable |= {key for key in set(value.memory_objects) | set(other.memory_objects)
                         if value.memory_objects.get(key) != other.memory_objects.get(key)}
            if value.whole != other.whole:
                unstable.add("#return_value")
    return frozenset(unstable)


@dataclass(frozen=True)
class ProbeCounts:
    """Q4 一趟的计数（读法 8、第六节 Q4-（ii）、Q4-（i）、Q4-对账）。"""

    probed: int
    counter_branch: int
    counter_branch_judged_legal: int
    counter_branch_judged_illegal: int
    undecidable_branch: int
    witness_refusals_without_either_branch: int
    counter_branch_mounted: int


WITNESS_REASON_ON_THE_LIST = "NewerPublishWitnessedBySystemConfiguration"


def probe_counts(run: RunDirectory, stream: str, batch: int) -> ProbeCounts:
    probes = [probe for probe in run.streams[stream].probes if probe.batch == batch]
    counter = [probe for probe in probes if probe.counter_branch_holds()]
    return ProbeCounts(
        probed=len(probes),
        counter_branch=len(counter),
        counter_branch_judged_legal=sum(probe.second_crop == "legal" for probe in counter),
        counter_branch_judged_illegal=sum(probe.second_crop.startswith("illegal") for probe in counter),
        undecidable_branch=sum(probe.undecidable_branch for probe in probes),
        witness_refusals_without_either_branch=sum(probe.refusal_on_the_d23_list == WITNESS_REASON_ON_THE_LIST
                                                   and not probe.counter_branch_holds() and not probe.undecidable_branch for probe in probes),
        counter_branch_mounted=sum(probe.mount == "mounted" for probe in counter),
    )


def q4_verdict(counter_branch: int, counter_branch_judged_legal: int) -> str:
    """Q4（第六节 Q4-（ii））：（ii）态数 0 → 甲；≥ 1 且全判不合法 → 乙；≥ 1 且至少一态判合法 → 丙。"""
    if counter_branch == 0:
        return "jia"
    return "bing" if counter_branch_judged_legal > 0 else "yi"


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


def trajectory(stream: StreamRun, stream_name: str, subcolumn_or_oracle: str, batch: int, o3: O3Verdicts | None,
               red_states_that_make_a_single_point: int = RED_STATES_OF_A_SINGLE_POINT) -> Trajectory:
    counts: dict[int, int] = {}
    first_red: StateRow | None = None
    for state in batch_states(stream, batch):
        counts.setdefault(state.segment, 0)
        if state_is_red(subcolumn_or_oracle, stream_name, state, o3):
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
        single_point_segments=tuple(segment for segment, count in ordered if count == red_states_that_make_a_single_point),
    )


def q5_verdict(lost: int, reference_filesystem_red: int, model_red: int, both_green: int, reference_filesystem_abstained: int,
               minimum_lost: int = 1) -> str:
    """Q5（第六节 Q5-有效、Q5-判）：L 少于 `minimum_lost`（登记写 L ≥ 1）→ F5 不够判；L 个态 RefFS 全弃权 → F8；任一子列 ≥ 1 态红 → 甲；
    都没红而至少一态两者都绿 → 乙；没有一态红、也没有一态两者都绿（全是弃权或 `judge_panic`）→ 同样照 F8 记不够判（修订 5 第 5.1 节第 7 条）。
    F8 那一支排在甲之前（RefFS 全弃权而模型崩溃判官有红，记 F8）：这是第一段比对器原有的次序，`test_segment_one.py` 钉住它。"""
    if lost < minimum_lost:
        return "not_enough_f5"
    if reference_filesystem_abstained == lost:
        return "not_enough_f8"
    if reference_filesystem_red > 0 or model_red > 0:
        return "jia"
    if both_green > 0:
        return "yi"
    return "not_enough_f8"


@dataclass(frozen=True)
class CountSeries:
    """一串按格号排开的计数（段号、第几次建 inode 发布、末段落了几个写）的三样（登记第八节；`test-discipline.md`「端点不是轨迹」）：
    峰值与它第一次出现的那一格（全 0 是 None）、为正的格数、末格（格号最大的那一格）的值。"""

    peak: int
    peak_at: int | None
    positive_cells: int
    last_cell_value: int


def count_series(cells: tuple[tuple[int, int], ...]) -> CountSeries:
    """`cells` 是 (格号, 计数)；该有而没有计数的格由调用方补成 0 交进来，末格才是登记说的那一格。"""
    ordered = sorted(cells)
    values = [count for _, count in ordered]
    peak = max(values, default=0)
    return CountSeries(peak=peak, peak_at=next((cell for cell, count in ordered if count == peak and count > 0), None),
                       positive_cells=sum(count > 0 for count in values), last_cell_value=values[-1] if values else 0)


def positive_cells_text(cells: tuple[tuple[int, int], ...]) -> str:
    """为正的格写成 `格:数,格:数`（按格号升序），没有写 `-`；为 0 的格不写，格的全集由同一行的格数字段给。"""
    return ",".join(f"{cell}:{count}" for cell, count in sorted(cells) if count > 0) or "-"


SECOND_CROP_LEGAL = "legal"


@dataclass(frozen=True)
class CounterBranchState:
    """Q4-（ii）逐态清单的一态（第六节 Q4-（ii））：探针那一行接回它的取样态。"""

    state: StateRow
    probe: ProbeRow


def counter_branch_states(stream: StreamRun, batch: int) -> list[CounterBranchState]:
    """这一批里（ii）计数那一支为真的态，按状态号；探针按状态号接回取样态，批号、序号对不上就抛错（不静默配错态）。"""
    states_by_number = {state.state_number: state for state in stream.states}
    found = []
    for probe in stream.probes:
        if probe.batch != batch or not probe.counter_branch_holds():
            continue
        found.append(CounterBranchState(probed_state(states_by_number, probe), probe))
    return sorted(found, key=lambda item: item.state.state_number)


def probed_state(states_by_number: dict[int, StateRow], probe: ProbeRow) -> StateRow:
    state = states_by_number.get(probe.state_number)
    if state is None or (state.batch, state.ordinal) != (probe.batch, probe.ordinal):
        raise ValueError(f"探针那一行接不回取样态：状态号 {probe.state_number}、批 {probe.batch}、序号 {probe.ordinal}")
    return state


def second_crop_verdict_and_unmet_condition(second_crop: str) -> tuple[str, str]:
    """第二截判定列拆成 (判定, 没满足的条件)：`illegal:<条件>` 的条件是第一个冒号之后的全部（条件里还带冒号）；
    `legal`、`reason_not_on_the_d23_list`、`-`（挂上了）没有条件，写 `-`。"""
    verdict, separator, condition = second_crop.partition(":")
    return verdict, (condition if separator else "-")


def counter_branch_by_segment(stream: StreamRun, batch: int) -> tuple[tuple[tuple[int, int], ...], tuple[tuple[int, int], ...]]:
    """第八节 Q4：这一批按段号排开的（ii）态数与其中第二截判合法的态数。格是这一批探过的态所在的每一段（没有（ii）的段记 0）。"""
    states_by_number = {state.state_number: state for state in stream.states}
    counter: dict[int, int] = {}
    legal: dict[int, int] = {}
    for probe in stream.probes:
        if probe.batch != batch:
            continue
        segment = probed_state(states_by_number, probe).segment
        counter.setdefault(segment, 0)
        legal.setdefault(segment, 0)
        if probe.counter_branch_holds():
            counter[segment] += 1
            legal[segment] += probe.second_crop == SECOND_CROP_LEGAL
    return tuple(sorted(counter.items())), tuple(sorted(legal.items()))
