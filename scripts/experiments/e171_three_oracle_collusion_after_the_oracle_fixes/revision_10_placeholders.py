#!/usr/bin/env python3
"""E171 第 1 次重跑·修订 10（修订 8 那一趟的第二段整表，登记 research/prompts/e171-r1-prereg.md 修订 10 与「主 agent 认定：修订 10 交回」）：
占位 H21–H29 的核对，连同第二段·修订 8 编排层要从修订 8 那一趟读的几样（起跑闸、T₂、本机闸、停派文件、逐行编排在不在跑）。
配置的形照同目录 revision_10_placeholders.json：H21–H29 一节一个，另有 revision_8_config 点名修订 8 那一趟的占位配置（revision_8_placeholders.json 那一形）。
两份都核：这一份核 H21–H29 与起跑闸；修订 8 那一份只核第二段读它的几项（tree、H3、H8、H16、H18），调 revision_8_placeholders.py 原来的核对函数（不改它）。
H21 每次现算一遍；H26、H29 是编排层跑的时候写下、现查的，核对时报 runtime。

起跑闸（修订 10 第 10.3 节依赖表第一行）：修订 8 那一趟第 0、1、2 步的单元绿；第 0 步产物的 research_side_differs 不是 false（读不到按 true）时，
  研究一侧变异表那一单元（research-mutations）也要绿。「绿」= 修订 8 的序列日志（<H16 日志目录>/sequence.log）里有那个单元的绿行，
  绿行点名的产物都在、行数与绿行记的相同（单元编排续跑那一套的前两样；清单 sha256 那一样归单元编排：它重跑的单元换 TAG，第二段在 run-values.tsv 里核得出）。
  没过退 78（前提没满足，不是停机，第 10.7 节运行条件末句），它们绿了同一条命令再起。
T₂（第 10.1 节，H21）：第 1 步绿的那一次的 TAG；计时产物 <H16 前缀>-s2-device-baseline-streams-1-2<TAG><H16 后缀>-timing.out 要在那条绿行点名的产物里；
  进程挂钟包住各流 elapsed_ns 之和（device_timing.py 同一判法），包不住不起第二段、交主 agent；逐行限时 max(1800, ⌈3 × T₂⌉)（驱动照 --baseline-timing 现算，
  这里用 device_timing.row_time_limit_seconds 算出来只写进日志）。
本机闸（第 10.3 节第 2 条）：第 7 步绿，且修订 8 序列日志里有「▶ 第 11–13 步：逐行编排起」那一行。
逐行编排在不在跑（让路用，H27）：修订 8 序列日志里最后一行「逐行编排起」在最后一行「逐行编排退」「收到停止信号」之后。
停派文件（第 10.3 节第 4 条）：<H16 日志目录>/stop-dispatching-rows（segment_one_units_revision_8.py 的 STOP_FILE）。

用法：
  revision_10_placeholders.py check --config C --phase <start|compare|contrast> [--repo R]
      start：起跑与每次续跑；compare：第二段比对之前（H22 与 TAG、step-9-z1 绿、H23）；contrast：出两趟第二段对照之前（H23、H28）。
      每项一行 name=placeholder …；末行 name=placeholders_verdict。
  revision_10_placeholders.py compare-command --config C [--repo R]
      打出第二段比对器那一条命令（segment_two_comparator.py 判法不改，输入换成这一趟的：第 1、7、9 步绿的那一次的导出、H23、树里的 crates/mutations.tsv）；不跑它。
  revision_10_placeholders.py --selftest [--cells 格,…] [--rounds N]
退出码：0 这一阶段要的都填了、都核过；78 有没填或核不过的（verdict 行点名哪几个；起跑闸没过也在这里）；2 用法错。

弄坏开关 PLACEHOLDERS_R10_BREAK=<项>（只在 --selftest 起的子进程里认，PLACEHOLDERS_R10_SELFTEST=1）。项名、弄坏哪一处、打开后红在哪一格：
  unfilled-passes                   没填（null）的占位当已填                          unfilled-named
  gate-ignores-research-mutations   起跑闸不看研究一侧变异表那一单元                  gate-research-mutations
  gate-ignores-line-counts          起跑闸不核绿行点名产物的行数                      gate-line-counts
  t2-cover-skipped                  T₂ 不核进程挂钟包住各流之和                        t2-covers-the-streams
  t2-any-product                    T₂ 不核计时产物在第 1 步绿行点名的产物里            t2-named-by-the-green-line
  local-gate-without-rows-start     本机闸只看第 7 步绿，不看逐行编排起了没有          local-gate
  running-ignores-end-lines         逐行编排在不在跑只看起过没有，不看退了没有          rows-orchestrator-running
  h23-collision-skipped             H23 不核与修订 6 第二段、修订 8 第一段撞名          h23-collision
  h24-old-names-accepted            H24 与修订 6 第二段、修订 8 第一段同名也收          h24-names
  h25-sha-skipped                   H25 不核各文件的 sha256                             h25-files
  h28-sha-skipped                   H28 不核文件的 sha256                               h28-files
  dry-run-accepted                  干跑配置在 start 阶段也算过                         dry-run-refused
"""
from __future__ import annotations

import argparse
import dataclasses
import json
import os
import shlex
import sys
import types

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import device_timing  # noqa: E402
import frozen_tree_lines_revision_8 as judge  # noqa: E402
import revision_8_placeholders as revision_8  # noqa: E402
import revision_8_selftest_harness as harness  # noqa: E402
import segment_one_units_revision_8 as units  # noqa: E402

BREAK_VARIABLE = "PLACEHOLDERS_R10_BREAK"
SELFTEST_VARIABLE = "PLACEHOLDERS_R10_SELFTEST"
EXIT_RUN_CONDITION = 78
PHASES = ("start", "compare", "contrast")
RESEARCH_RELATIVE = revision_8.RESEARCH_RELATIVE
SEGMENT_TWO_SUFFIX = "-r8"
NANOSECONDS_PER_SECOND = 1_000_000_000
# 修订 6 那一趟第二段用过的（run_segment_two_on_the_revision_6_tree_multi_host.sh 与 rows_orchestrator_segment_two.py）：导出根、草稿目录（它的日志与工作根都在下面）、
# 别的台四个落点；两段的标签前缀（修订 6 第二段 e171s2-、修订 8 第一段 e171s1r8）
REVISION_6_SEGMENT_TWO_DUMPS = "scripts/experiments/target/e171-r1-dumps/revision-6-segment-2"
REVISION_6_SEGMENT_TWO_SCRATCH = os.path.join(revision_8.WORK_ROOT, "e171-r1-segment2-multi-host")
REVISION_6_SEGMENT_TWO_PEER_NAMES = ("research/e171-s1-peer-kit", "research/e171-s1-frozen-snapshot", "scripts/experiments/target/e171-s1-peer-out", "research/e171-s2-launch")
OTHER_LABEL_PREFIXES = ("e171s2-", "e171s1r8")
PEER_NAME_KEYS = ("peer_kit", "peer_snapshot", "peer_out", "peer_launch")
STEP_1_STEM = "s2-device-baseline-streams-1-2"
GATE_UNITS = ("step-0", "step-1", "step-2")
RESEARCH_MUTATIONS_UNIT = "research-mutations"
ROWS_ORCHESTRATOR_STARTED = "▶ 第 11–13 步：逐行编排起"
ROWS_ORCHESTRATOR_ENDED = ("逐行编排退", "收到停止信号")
# 比对器的三份基线导出（第 10.1 节）：第 1 步那一份是 Q2 的基线，第 7、9 步两份是读法 4 的稳定性基线
COMPARE_BASELINES = (("baseline", "step-1", STEP_1_STEM), ("stability_baseline_run1", "step-7", "baseline-run1"),
                     ("stability_baseline_run2", "step-9", "baseline-run2"))
# H25：编排层、入口、对照脚本，连同它们按 sha256 载入或 import 的几份（比对器不改，不在里面）
H25_REQUIRED_FILES = tuple(f"{RESEARCH_RELATIVE}/{name}" for name in (
    "run_segment_two_revision_10_multi_host.sh", "rows_orchestrator_segment_two_revision_10.py", "revision_10_placeholders.py",
    "contrast_segment_two_revision_10.py", "revision_8_selftest_harness.py", "rows_orchestrator_segment_two.py", "rows_orchestrator.py"))


def broken(item: str) -> bool:
    return harness.broken(BREAK_VARIABLE, item)


def segment_two_prefix(run_date: str) -> str:
    return f"{revision_8.PRODUCT_STEM}-{run_date}-s2"


@dataclasses.dataclass(frozen=True)
class SegmentOne:
    """修订 8 那一趟第一段（第二段只读它）：树、H3、H8 与 H16 那几样。"""
    tree: str
    snapshot_digest: str
    mutations_sha256: str
    run_date: str
    suffix: str
    dumps: str
    logs: str

    @property
    def prefix(self) -> str:
        return revision_8.product_prefix(self.run_date)

    def product(self, stem: str) -> str:
        return f"{self.prefix}-{stem}{self.suffix}.out"

    def timing(self, stem: str) -> str:
        return f"{self.prefix}-{stem}{self.suffix}-timing.out"

    @property
    def sequence_log(self) -> str:
        return os.path.join(self.logs, "sequence.log")

    @property
    def stop_file(self) -> str:
        return os.path.join(self.logs, units.STOP_FILE)


@dataclasses.dataclass(frozen=True)
class SegmentTwo:
    """这一段：H23（日期、后缀、导出根、日志目录、工作根）与 H24（别的台四个落点、标签前缀）。"""
    run_date: str
    suffix: str
    dumps: str
    logs: str
    work_root: str
    peer_names: tuple[str, ...]
    label_prefix: str

    @property
    def prefix(self) -> str:
        return segment_two_prefix(self.run_date)

    @property
    def rows_root(self) -> str:
        return os.path.join(self.dumps, "segment2-rows")


@dataclasses.dataclass(frozen=True)
class BaselineTimingOfStepOne:
    """T₂（H21）：第 1 步绿的那一次的 TAG、计时产物（相对仓根）与它的 sha256、进程挂钟、各流之和、逐行限时。"""
    tag: str
    path: str
    sha256: str
    seconds: float
    stream_seconds: float
    row_time_limit_seconds: int


# ── 读两份配置与修订 8 那一趟的序列日志（编排层与核对共用；都是纯读） ──
def load_configs(repo: str, config_path: str) -> tuple[dict, dict]:
    """交回 (这一份配置, 修订 8 那一份配置)；读不到退 78 那一类。"""
    config = judge.load_config(config_path)
    revision_8_path = config.get("revision_8_config")
    if not isinstance(revision_8_path, str):
        raise judge.RunConditionError(f"{config_path} 没写 revision_8_config（修订 8 那一趟的占位配置）")
    return config, judge.load_config(judge.resolve(repo, revision_8_path))


def segment_one_of(revision_8_config: dict) -> SegmentOne:
    h16 = judge.section(revision_8_config, "H16")
    return SegmentOne(tree=judge.tree_of(revision_8_config),
                      snapshot_digest=str(judge.section(revision_8_config, "H3").get("snapshot_digest") or "unfilled"),
                      mutations_sha256=str(judge.section(revision_8_config, "H8").get("mutations_table_sha256") or "unfilled"),
                      run_date=str(h16.get("run_date") or "unfilled"), suffix=str(h16.get("run_suffix") or SEGMENT_TWO_SUFFIX),
                      dumps=str(h16.get("dumps_root") or "unfilled"), logs=str(h16.get("logs_directory") or "unfilled"))


def segment_two_of(config: dict) -> SegmentTwo:
    h23, h24 = judge.section(config, "H23"), judge.section(config, "H24")
    return SegmentTwo(run_date=str(h23.get("run_date") or "unfilled"), suffix=str(h23.get("run_suffix") or SEGMENT_TWO_SUFFIX),
                      dumps=str(h23.get("dumps_root") or "unfilled"), logs=str(h23.get("logs_directory") or "unfilled"),
                      work_root=str(h23.get("work_root") or "unfilled"), peer_names=tuple(str(h24.get(key)) for key in PEER_NAME_KEYS),
                      label_prefix=str(h24.get("label_prefix")))


def green_units(segment_one: SegmentOne) -> dict[str, tuple[int, str, list[tuple[str, int]]]]:
    """修订 8 序列日志里每个单元最后一条绿行（单元编排续跑用的同一个读法）：单元 → (第几次, TAG, [(产物, 行数)])。"""
    return units.green_records(types.SimpleNamespace(logs=segment_one.logs))


def sequence_lines(segment_one: SegmentOne) -> list[str]:
    path = segment_one.sequence_log
    return open(path, encoding="utf-8").read().splitlines() if os.path.isfile(path) else []


def unit_problems(repo: str, records: dict, unit: str) -> list[str]:
    """一个单元算不算绿：没有绿行、绿行点名的产物不在或行数不同，各记一条。"""
    if unit not in records:
        return [f"{unit}:no_green_line"]
    problems = []
    for relative, lines in records[unit][2]:
        path = judge.resolve(repo, relative)
        if not os.path.isfile(path):
            problems.append(f"{unit}:product_missing:{os.path.basename(relative)}")
        elif units.line_count(path) != lines and not broken("gate-ignores-line-counts"):
            problems.append(f"{unit}:line_count:{os.path.basename(relative)}")
    return problems


def research_mutations_required(repo: str, segment_one: SegmentOne, records: dict) -> bool:
    """第 0 步产物 frozen-tree<TAG> 的 verdict 行 research_side_differs 不是 false 就要等研究一侧变异表（读不到按 true，同单元编排）。"""
    if broken("gate-ignores-research-mutations"):
        return False
    if "step-0" not in records:
        return True
    path = judge.resolve(repo, segment_one.product(f"frozen-tree{records['step-0'][1]}"))
    return not os.path.isfile(path) or judge.verdict_fields(path).get("research_side_differs", "true") != "false"


def start_gate_problems(repo: str, segment_one: SegmentOne) -> list[str]:
    """起跑闸：空表就是第 0、1、2 步（与研究一侧变异表，排了的话）都绿。"""
    records = green_units(segment_one)
    wanted = GATE_UNITS + ((RESEARCH_MUTATIONS_UNIT,) if research_mutations_required(repo, segment_one, records) else ())
    return [problem for unit in wanted for problem in unit_problems(repo, records, unit)]


def resolve_baseline_timing(repo: str, segment_one: SegmentOne) -> BaselineTimingOfStepOne:
    """T₂（文件头）；取不到或包不住各流之和抛 judge.RunConditionError。"""
    records = green_units(segment_one)
    if "step-1" not in records:
        raise judge.RunConditionError("修订 8 那一趟第 1 步还没有绿行，T₂ 取不到")
    _, tag, products = records["step-1"]
    relative = segment_one.timing(f"{STEP_1_STEM}{tag}")
    if relative not in {product for product, _ in products} and not broken("t2-any-product"):
        raise judge.RunConditionError(f"T₂ 的计时产物 {relative} 不在第 1 步绿行（TAG {tag or '-'}）点名的产物里")
    path = judge.resolve(repo, relative)
    if not os.path.isfile(path):
        raise judge.RunConditionError(f"T₂ 的计时产物不在：{relative}")
    timing = device_timing.read_baseline_timing(open(path, encoding="utf-8").read().splitlines())
    covers = timing.contains_the_stream_times() or (broken("t2-cover-skipped") and timing.process_wall_seconds is not None)
    stream_seconds = timing.stream_time_sum_nanoseconds / NANOSECONDS_PER_SECOND
    if not covers:
        raise judge.RunConditionError(f"T₂ 的进程挂钟没包住各流 elapsed_ns 之和（{relative}：process_wall={timing.process_wall_seconds}，"
                                      f"各流之和 {stream_seconds:.3f} 秒，{timing.streams} 条流）：不起第二段，交主 agent（第 10.1 节）")
    return BaselineTimingOfStepOne(tag, relative, judge.sha256_of_file(path), timing.process_wall_seconds, stream_seconds,
                                   device_timing.row_time_limit_seconds(timing.process_wall_seconds, False))


def local_gate(segment_one: SegmentOne) -> tuple[bool, str]:
    """本机闸：(开没开, 说明)。"""
    step_7_green = "step-7" in green_units(segment_one)
    rows_started = any(ROWS_ORCHESTRATOR_STARTED in line for line in sequence_lines(segment_one))
    is_open = step_7_green and (rows_started or broken("local-gate-without-rows-start"))
    return is_open, f"第 7 步绿 {str(step_7_green).lower()}、修订 8 逐行编排起过 {str(rows_started).lower()}"


def rows_orchestrator_running(lines: list[str]) -> bool:
    """修订 8 的逐行编排此刻在不在跑：最后一行「逐行编排起」在最后一行「逐行编排退」「收到停止信号」之后。"""
    last_start = max((index for index, line in enumerate(lines) if ROWS_ORCHESTRATOR_STARTED in line), default=-1)
    if broken("running-ignores-end-lines"):
        return last_start >= 0
    last_end = max((index for index, line in enumerate(lines) if any(marker in line for marker in ROWS_ORCHESTRATOR_ENDED)), default=-1)
    return last_start > last_end


def segment_one_cross_host_verdict(repo: str, segment_one: SegmentOne) -> tuple[str, str]:
    """S17 在第一段的那一处（第 10.7 节）：交回 (产物 <H16 前缀>-rows-cross-host-check<H16 后缀>.out, 它 verdict 行的 cross_host_identical；
    产物不在或没有 verdict 行写 absent)。"""
    relative = segment_one.product("rows-cross-host-check")
    path = judge.resolve(repo, relative)
    return relative, judge.verdict_fields(path).get("cross_host_identical", "absent") if os.path.isfile(path) else "absent"


def compare_baselines(segment_one: SegmentOne) -> dict[str, str]:
    """比对器的三份基线导出（相对仓根）：H16 导出根下、各自那一步绿的那一次的 TAG；那一步没绿的不在表里。"""
    records = green_units(segment_one)
    return {key: os.path.join(segment_one.dumps, f"{stem}{records[unit][1]}") for key, unit, stem in COMPARE_BASELINES if unit in records}


# ── 核对：每个占位一行 ──
Outcome = revision_8.Outcome
RUN_VALUES_FILE = "run-values.tsv"
INPUTS_FILE = "inputs.sha256"
RUNTIME_PLACEHOLDERS = {
    "H26": f"编排层在第一趟开跑那一刻写进 <H23 导出根>/{INPUTS_FILE} 与 {RUN_VALUES_FILE}，续跑先核（对不上停，退 1）",
    "H29": "编排层每次起跑、续跑、加槽现查现算（第二段日志里「分配」「加槽」那几行）",
}


def absent(values: dict, keys: tuple[str, ...]) -> list[str]:
    if broken("unfilled-passes"):
        return []
    return [key for key in keys if values.get(key) is None]


def overlapping(first: str, second: str) -> bool:
    """两个目录是同一个、或一个在另一个里面。"""
    first, second = os.path.normpath(first), os.path.normpath(second)
    return first == second or first.startswith(second + os.sep) or second.startswith(first + os.sep)


def revision_8_items(repo: str, revision_8_config: dict, phase: str) -> list[tuple[str, Outcome]]:
    """修订 8 那一份里第二段读的几项，调修订 8 的核对函数：start 照它自己的 start；别的阶段只核树的摘要（它的 contrast 那一形）与 H16 的形状。"""
    if phase == "start":
        return [("revision_8.tree", revision_8.check_tree(repo, revision_8_config, "start")),
                ("revision_8.H3", revision_8.check_h3(repo, revision_8_config, "start")),
                ("revision_8.H8", revision_8.check_h8(repo, revision_8_config, "start")),
                ("revision_8.H16", revision_8.check_h16(repo, revision_8_config, "start")),
                ("revision_8.H18", revision_8.check_h18(repo, revision_8_config, "start"))]
    return [("revision_8.H3", revision_8.check_h3(repo, revision_8_config, "contrast")),
            ("revision_8.H16", revision_8.check_h16(repo, revision_8_config, "contrast"))]


def check_h21(repo: str, config: dict, revision_8_config: dict, phase: str) -> Outcome:
    try:
        timing = resolve_baseline_timing(repo, segment_one_of(revision_8_config))
    except judge.RunConditionError as error:
        return Outcome(True, False, str(error))
    return Outcome(True, True, f"TAG {timing.tag or '-'}；{timing.path}（sha256 {timing.sha256[:16]}…）；process_wall {timing.seconds:.3f} 秒包住各流之和 "
                               f"{timing.stream_seconds:.3f} 秒；逐行限时 {timing.row_time_limit_seconds} 秒")


def check_gate(repo: str, config: dict, revision_8_config: dict, phase: str) -> Outcome:
    problems = start_gate_problems(repo, segment_one_of(revision_8_config))
    return Outcome(True, not problems, f"修订 8 那一趟第 0、1、2 步（与研究一侧变异表，排了的话）都绿：没过的 {','.join(problems) or '无'}（没过退 78，绿了再起）")


def check_h22(repo: str, config: dict, revision_8_config: dict, phase: str) -> Outcome:
    h22 = judge.section(config, "H22")
    missing = absent(h22, tuple(key for key, _, _ in COMPARE_BASELINES))
    if missing:
        return Outcome(False, False, "没填：" + ",".join(missing))
    segment_one = segment_one_of(revision_8_config)
    expected = compare_baselines(segment_one)
    same = all(h22.get(key) == expected.get(key) for key, _, _ in COMPARE_BASELINES)
    present = all(os.path.isdir(judge.resolve(repo, str(h22[key]))) for key, _, _ in COMPARE_BASELINES)
    records = green_units(segment_one)
    z1_green = not unit_problems(repo, records, "step-9-z1")
    return Outcome(True, same and present and z1_green, f"三份基线与第 1、7、9 步绿的那一次（TAG）对得上 {str(same).lower()}、都在 {str(present).lower()}；"
                                                        f"step-9-z1 绿 {str(z1_green).lower()}")


def check_h23(repo: str, config: dict, revision_8_config: dict, phase: str) -> Outcome:
    h23 = judge.section(config, "H23")
    missing = absent(h23, ("run_date", "run_suffix", "dumps_root", "logs_directory", "work_root"))
    if missing:
        return Outcome(False, False, "没填：" + ",".join(missing))
    segment_one, segment_two = segment_one_of(revision_8_config), segment_two_of(config)
    work = revision_8.WORK_ROOT
    shapes = (bool(revision_8.DATE.fullmatch(segment_two.run_date)) and (bool(config.get("dry_run")) or segment_two.suffix == SEGMENT_TWO_SUFFIX)
              and segment_two.dumps.startswith(revision_8.DUMPS_PARENT)
              and all(os.path.isabs(path) and os.path.abspath(path).startswith(work) for path in (segment_two.logs, segment_two.work_root)))
    clashes = []
    if not broken("h23-collision-skipped"):
        if segment_two.dumps.startswith(REVISION_6_SEGMENT_TWO_DUMPS) or overlapping(segment_two.dumps, segment_one.dumps):
            clashes.append(f"导出根 {segment_two.dumps}")
        clashes += [path for path in (segment_two.logs, segment_two.work_root)
                    if overlapping(path, REVISION_6_SEGMENT_TWO_SCRATCH) or overlapping(path, segment_one.logs)]
        first_start = not os.path.exists(os.path.join(judge.resolve(repo, segment_two.dumps), RUN_VALUES_FILE))
        if first_start and phase == "start":
            results = os.path.join(repo, "research/results")
            prefix = os.path.basename(segment_two.prefix) + "-"
            clashes += [name for name in (os.listdir(results) if os.path.isdir(results) else []) if name.startswith(prefix)
                        and (name.endswith(f"{segment_two.suffix}.out") or name.endswith(f"{segment_two.suffix}-timing.out"))]
            if os.path.exists(judge.resolve(repo, segment_two.dumps)):
                clashes.append(f"导出根已存在 {segment_two.dumps}")
    return Outcome(True, shapes and not clashes, f"日期、后缀 {segment_two.suffix}、导出根在 {revision_8.DUMPS_PARENT} 下、日志与工作根在工作根下 {str(shapes).lower()}；"
                                                 f"与修订 6 第二段、修订 8 第一段撞名或重叠的 {len(clashes)} 个{('：' + '，'.join(clashes[:4])) if clashes else ''}")


def check_h24(repo: str, config: dict, revision_8_config: dict, phase: str) -> Outcome:
    h24 = judge.section(config, "H24")
    missing = absent(h24, PEER_NAME_KEYS + ("label_prefix",))
    if missing:
        return Outcome(False, False, "没填：" + ",".join(missing))
    names, label = tuple(str(h24[key]) for key in PEER_NAME_KEYS), str(h24["label_prefix"])
    taken = set(REVISION_6_SEGMENT_TWO_PEER_NAMES) | {str(value) for value in judge.section(revision_8_config, "H18").values()}
    clash = [] if broken("h24-old-names-accepted") else [name for name in names if name in taken]
    label_clash = [] if broken("h24-old-names-accepted") else [other for other in OTHER_LABEL_PREFIXES
                                                                  if f"{label}-".startswith(other) or other.startswith(label)]
    shapes = (all(name.startswith("research/") for name in names) and len(set(names)) == len(names) and names[2].startswith("scripts/experiments/target/")
              and bool(label) and all(character.isalnum() or character in "._-" for character in label))
    return Outcome(True, shapes and not clash and not label_clash, f"四个落点在 research/ 下、互不相同、取回的在 scripts/experiments/target/ 下、标签前缀能当标签 {str(shapes).lower()}；"
                                                                   f"与修订 6 第二段、修订 8 第一段同名的落点 {clash or '无'}、标签前缀相撞的 {label_clash or '无'}")


def check_h25(repo: str, config: dict, revision_8_config: dict, phase: str) -> Outcome:
    files = judge.section(config, "H25").get("files")
    if files is None and not broken("unfilled-passes"):
        return Outcome(False, False, "没填")
    files = files or {}
    missing = [path for path in H25_REQUIRED_FILES if path not in files]
    differing = [] if broken("h25-sha-skipped") else [path for path, expected in files.items() if judge.sha256_or_missing(judge.resolve(repo, path)) != expected]
    return Outcome(True, not missing and not differing, f"登了 {len(files)} 份；该登没登的 {len(missing)} 份；sha256 与现在不同的 {','.join(os.path.basename(path) for path in differing) or '无'}")


def check_h27(repo: str, config: dict, revision_8_config: dict, phase: str) -> Outcome:
    h27 = judge.section(config, "H27")
    missing = absent(h27, ("yield_implemented", "how"))
    if missing:
        return Outcome(False, False, "没填：" + ",".join(missing))
    good = isinstance(h27["yield_implemented"], bool) and len(str(h27["how"])) >= 8
    return Outcome(True, good, f"让路{'补上了' if h27['yield_implemented'] is True else '没补'}（第 10.3 节第 3 条）：{str(h27['how'])[:60]}")


H28_FILE_KEYS = ("comparator_product", "driver_product", "driver_timing", "rows_hosts_ledger")


def check_h28(repo: str, config: dict, revision_8_config: dict, phase: str) -> Outcome:
    h28 = judge.section(config, "H28")
    missing = absent(h28, H28_FILE_KEYS + ("rows_root", "product_prefix", "product_suffix", "sha256"))
    if missing:
        return Outcome(False, False, "没填：" + ",".join(missing))
    recorded = h28["sha256"] if isinstance(h28["sha256"], dict) else {}
    paths = [str(h28[key]) for key in H28_FILE_KEYS]
    present = all(os.path.isfile(judge.resolve(repo, path)) for path in paths) and os.path.isdir(judge.resolve(repo, str(h28["rows_root"])))
    differing = [] if broken("h28-sha-skipped") else [path for path in paths if recorded.get(path) != judge.sha256_or_missing(judge.resolve(repo, path))]
    return Outcome(True, present and not differing, f"修订 6 那一趟第二段的四个文件与行输出根都在 {str(present).lower()}；sha256 没记或与现在不同的 "
                                                    f"{','.join(os.path.basename(path) for path in differing) or '无'}")


def check_dry_run(repo: str, config: dict, revision_8_config: dict, phase: str) -> Outcome:
    dry = bool(config.get("dry_run")) or bool(revision_8_config.get("dry_run"))
    if phase == "start" and dry and not broken("dry-run-accepted"):
        return Outcome(True, False, f"这是干跑配置（{config.get('dry_run_note') or revision_8_config.get('dry_run_note') or '没写说明'}）：只给 PLAN_ONLY 用，不许真起跑")
    return Outcome(True, True, "干跑配置" if dry else "不是干跑配置")


@dataclasses.dataclass(frozen=True)
class Placeholder:
    key: str
    what: str
    who: str
    when: str
    phases: tuple[str, ...]
    check: object


PLACEHOLDERS = (
    Placeholder("dry_run", "两份配置都不是干跑用的", "模板写死", "—", ("start",), check_dry_run),
    Placeholder("gate", "起跑闸：修订 8 那一趟第 0、1、2 步（与研究一侧变异表，排了的话）绿", "修订 8 的单元编排", "第二段第一行之前", ("start",), check_gate),
    Placeholder("H21", "T₂：第 1 步绿的那一次的 TAG、计时产物与 sha256、process_wall、逐行限时", "编排层现取", "第 0、1、2 步绿之后、第一行之前", ("start",), check_h21),
    Placeholder("H22", "比对器的三份基线导出（第 1、7、9 步绿的那一次）", "接第二段比对的执行员", "比对之前", ("compare",), check_h22),
    Placeholder("H23", "第二段的 RUN_DATE、后缀、导出根、日志目录、工作根（产物前缀 …-<RUN_DATE>-s2、行输出根 <导出根>/segment2-rows 由它们定）", "主 agent",
                "起跑时", ("start", "compare", "contrast"), check_h23),
    Placeholder("H24", "别的台四个落点与标签前缀", "执行员", "写跑法时", ("start",), check_h24),
    Placeholder("H25", "编排层、入口、对照脚本与它们载入的几份的 sha256", "执行员写完，主 agent 认", "起跑之前", ("start",), check_h25),
    Placeholder("H27", "让路补没补上", "执行员写，主 agent 认", "起跑之前", ("start",), check_h27),
    Placeholder("H28", "修订 6 那一趟第二段拿来对照的产物与导出", "主 agent", "出对照产物之前", ("contrast",), check_h28),
)


def check_config(repo: str, config: dict, revision_8_config: dict, phase: str) -> tuple[list[str], list[str], list[str]]:
    """这一阶段要的每一项：打一行，交回 (没填的, 核不过的, 打出的行)。"""
    unfilled_keys, failing, printed = [], [], []
    items = [(key, outcome, "修订 8 的核对函数", "照修订 8 第 8.8 节", "第二段读修订 8 那一份的这一项")
             for key, outcome in revision_8_items(repo, revision_8_config, phase)]
    items += [(item.key, item.check(repo, config, revision_8_config, phase), item.who, item.when, item.what) for item in PLACEHOLDERS if phase in item.phases]
    for key, outcome, who, when, what in items:
        if not outcome.filled:
            unfilled_keys.append(key)
        elif not outcome.holds:
            failing.append(key)
        printed.append(f"name=placeholder key={key} phase={phase} filled={str(outcome.filled).lower()} holds={str(outcome.holds).lower()} "
                       f"who={who} when={when} what={what} detail={outcome.detail}")
    if phase == "start":
        printed += [f"name=placeholder key={key} phase={phase} filled=runtime where={where}" for key, where in RUNTIME_PLACEHOLDERS.items()]
    return unfilled_keys, failing, printed


def compare_command(repo: str, config: dict, revision_8_config: dict) -> list[str]:
    """第二段比对器那一条命令（仓根起）：输入换成这一趟的，判法不改（第 10.4 节）。三份基线有一份那一步还没绿就抛 RunConditionError。"""
    segment_one, segment_two = segment_one_of(revision_8_config), segment_two_of(config)
    baselines = compare_baselines(segment_one)
    missing = [key for key, _, _ in COMPARE_BASELINES if key not in baselines]
    if missing:
        raise judge.RunConditionError(f"基线那一步还没绿：{','.join(missing)}")
    return ["python3", "-B", f"{RESEARCH_RELATIVE}/segment_two_comparator.py", "--baseline", baselines["baseline"],
            "--stability-baseline", baselines["stability_baseline_run1"], "--stability-baseline", baselines["stability_baseline_run2"],
            "--rows-root", segment_two.rows_root, "--driver-product", f"{segment_two.prefix}-rows-driver{segment_two.suffix}.out",
            "--mutations-table", os.path.join(segment_one.tree, judge.MUTATIONS_TABLE), "--dumps-root", segment_two.dumps, "--dumps-root", segment_one.dumps,
            "--product-prefix", os.path.basename(segment_two.prefix) + "-", f"--product-suffix={segment_two.suffix}.out",
            "--output", f"{segment_two.prefix}-comparator{segment_two.suffix}.out"]


def main(argument_list: list[str]) -> int:
    refusal = harness.refuse_break_outside_selftest(BREAK_VARIABLE, SELFTEST_VARIABLE)
    if refusal:
        print(refusal, file=sys.stderr)
        return 2
    if argument_list[:1] == ["--selftest"]:
        return selftest(argument_list[1:])
    parser = argparse.ArgumentParser(description="E171 修订 10：占位 H21–H29 的核对与第二段比对器那一条命令")
    parser.add_argument("mode", choices=("check", "compare-command"))
    parser.add_argument("--config", required=True)
    parser.add_argument("--phase", choices=PHASES)
    parser.add_argument("--repo", default=os.getcwd())
    arguments = parser.parse_args(argument_list)
    repo = os.path.abspath(arguments.repo)
    try:
        config, revision_8_config = load_configs(repo, arguments.config)
        if arguments.mode == "compare-command":
            print(" ".join(shlex.quote(part) for part in compare_command(repo, config, revision_8_config)))
            return 0
    except judge.RunConditionError as error:
        print(f"✗ {error}")
        return EXIT_RUN_CONDITION
    if arguments.phase is None:
        parser.error("check 要给 --phase")
    unfilled_keys, failing, printed = check_config(repo, config, revision_8_config, arguments.phase)
    for line in printed:
        print(line)
    print(f"name=placeholders_verdict phase={arguments.phase} config={arguments.config} config_sha256={judge.sha256_of_file(arguments.config)} "
          f"dry_run={str(bool(config.get('dry_run'))).lower()} unfilled={','.join(unfilled_keys) or 'none'} failing={','.join(failing) or 'none'} "
          f"runtime={','.join(RUNTIME_PLACEHOLDERS) if arguments.phase == 'start' else 'none'}")
    return EXIT_RUN_CONDITION if unfilled_keys or failing else 0


# ── 自证：假的两份配置、假的修订 8 序列日志与产物（不读真的仓；H25 那一格读真的仓里那几份文件） ──
FAKE_RUN_DATE = "2026-10-09"
FAKE_NEVER_CREATED = os.path.join(revision_8.WORK_ROOT, "e171-r10-selftest-never-created")


def template() -> dict:
    with open(os.path.join(HERE, "revision_10_placeholders.json"), encoding="utf-8") as handle:
        return json.load(handle)


def fake_segment_one(scratch: str) -> SegmentOne:
    return SegmentOne(tree="scripts/experiments/target/fake-tree", snapshot_digest="d" * 64, mutations_sha256="e" * 64, run_date=FAKE_RUN_DATE, suffix="-r8",
                      dumps="scripts/experiments/target/e171-r1-dumps/fake-segment-one", logs=os.path.join(scratch, "segment-one-logs"))


def green_line(unit: str, products: list[str], counts: dict[str, int], tag: str = "-") -> str:
    return f"{FAKE_RUN_DATE} ✓ 单元 {unit} 绿 attempt=1 tag={tag}" + "".join(f" 产物 {path}（{counts[path]} 行）" for path in products)


def fake_world(scratch: str, research_side_differs: str = "false", step_1_line_count: int | None = None, process_wall: str = "700.200",
               timing_named: bool = True, extra_lines: tuple[str, ...] = ()) -> tuple[str, dict, dict]:
    """交回 (假仓, 这一份配置, 修订 8 那一份配置)：修订 8 那一趟第 0、1、2 步绿；第 1 步的计时产物两条流共 70 秒。"""
    repo = os.path.join(scratch, "repo")
    segment_one = fake_segment_one(scratch)
    frozen, product, timing = segment_one.product("frozen-tree"), segment_one.product(STEP_1_STEM), segment_one.timing(STEP_1_STEM)
    judge.write_text(os.path.join(repo, frozen), f"E7RESULT name=verdict research_side_differs={research_side_differs} triggered=none\n")
    judge.write_text(os.path.join(repo, product), "E7RESULT name=verdict s1_failed_checks=0\n")
    judge.write_text(os.path.join(repo, timing), "E7TIMING name=stream_time stream=a elapsed_ns=40000000000\nE7TIMING name=stream_time stream=b elapsed_ns=30000000000\n"
                                                 f"E7TIMING name=process_wall seconds={process_wall} exit=0\n")
    counts = {path: units.line_count(os.path.join(repo, path)) for path in (frozen, product, timing)}
    if step_1_line_count is not None:
        counts[product] = step_1_line_count
    lines = [green_line("step-0", [frozen], counts), green_line("step-1", [product] + ([timing] if timing_named else []), counts),
             green_line("step-2", [], counts)] + list(extra_lines)
    judge.write_text(segment_one.sequence_log, "\n".join(lines) + "\n")
    revision_8_config = {"tree": segment_one.tree, "H3": {"snapshot_digest": segment_one.snapshot_digest}, "H8": {"mutations_table_sha256": segment_one.mutations_sha256},
                         "H16": {"run_date": FAKE_RUN_DATE, "run_suffix": "-r8", "dumps_root": segment_one.dumps, "logs_directory": segment_one.logs},
                         "H18": revision_8.template()["H18"]}
    config = template()
    config["H23"] = {"run_date": FAKE_RUN_DATE, "run_suffix": "-r8", "dumps_root": "scripts/experiments/target/e171-r1-dumps/fake-segment-two",
                     "logs_directory": os.path.join(FAKE_NEVER_CREATED, "logs"), "work_root": os.path.join(FAKE_NEVER_CREATED, "work")}
    return repo, config, revision_8_config


def cell_template_shape(scratch: str) -> tuple[bool, str]:
    config = template()
    keys = {f"H{number}" for number in range(21, 30)}
    present = sorted(key for key in keys if key in config)
    good = set(present) == keys and config.get("dry_run") is False and isinstance(config.get("revision_8_config"), str)
    return good, f"模板里有的占位 {len(present)} 个（要 9），dry_run={config.get('dry_run')}，revision_8_config={config.get('revision_8_config')}"


def cell_unfilled_named(scratch: str) -> tuple[bool, str]:
    unfilled_keys, failing, _ = check_config(scratch, template(), revision_8.template(), "start")
    return {"H23", "H25"} <= set(unfilled_keys), f"两份模板在 start 阶段点名的没填 {unfilled_keys}（要含 H23、H25），核不过的 {failing}"


def cell_gate_research_mutations(scratch: str) -> tuple[bool, str]:
    repo, _, revision_8_config = fake_world(os.path.join(scratch, "differs"), research_side_differs="true")
    differs = start_gate_problems(repo, segment_one_of(revision_8_config))
    repo, _, revision_8_config = fake_world(os.path.join(scratch, "same"))
    same = start_gate_problems(repo, segment_one_of(revision_8_config))
    return differs == ["research-mutations:no_green_line"] and same == [], f"第 0 步报有不同时没过的 {differs}（要研究一侧变异表没绿）；报没有不同时 {same}"


def cell_gate_line_counts(scratch: str) -> tuple[bool, str]:
    repo, _, revision_8_config = fake_world(scratch, step_1_line_count=5)
    problems = start_gate_problems(repo, segment_one_of(revision_8_config))
    return [problem.split(":")[:2] for problem in problems] == [["step-1", "line_count"]], f"绿行记 5 行、产物 1 行：没过的 {problems}"


def cell_t2_covers_the_streams(scratch: str) -> tuple[bool, str]:
    repo, _, revision_8_config = fake_world(os.path.join(scratch, "good"))
    good = resolve_baseline_timing(repo, segment_one_of(revision_8_config))
    repo, _, revision_8_config = fake_world(os.path.join(scratch, "short"), process_wall="69.000")
    try:
        short = resolve_baseline_timing(repo, segment_one_of(revision_8_config))
        refused = f"收了 {short.seconds}"
    except judge.RunConditionError as error:
        refused = ""
        reason = str(error)
    ok = good.seconds == 700.2 and good.row_time_limit_seconds == 2101 and good.stream_seconds == 70.0 and not refused
    return ok, f"700.2 秒包住 70 秒：限时 {good.row_time_limit_seconds}（要 2101）；69 秒包不住 70 秒：{refused or '拒（' + reason[:40] + '）'}"


def cell_t2_named_by_the_green_line(scratch: str) -> tuple[bool, str]:
    repo, _, revision_8_config = fake_world(scratch, timing_named=False)
    try:
        resolve_baseline_timing(repo, segment_one_of(revision_8_config))
    except judge.RunConditionError as error:
        return True, f"绿行没点名计时产物：拒（{str(error)[:60]}）"
    return False, "绿行没点名计时产物却收了"


def cell_local_gate(scratch: str) -> tuple[bool, str]:
    counts: dict[str, int] = {}
    step_7 = green_line("step-7", [], counts)
    _, _, closed_config = fake_world(os.path.join(scratch, "closed"), extra_lines=(step_7,))
    _, _, open_config = fake_world(os.path.join(scratch, "open"), extra_lines=(step_7, f"{FAKE_RUN_DATE} {ROWS_ORCHESTRATOR_STARTED}（step-0 都绿）"))
    closed, open_ = local_gate(segment_one_of(closed_config)), local_gate(segment_one_of(open_config))
    return not closed[0] and open_[0], f"只有第 7 步绿：{closed}；再加逐行编排起：{open_}"


def cell_rows_orchestrator_running(scratch: str) -> tuple[bool, str]:
    start, end, signal_line = f"x {ROWS_ORCHESTRATOR_STARTED}（…）", "x   逐行编排退 0", "x 收到停止信号：给在跑的单元与逐行编排各发一次 TERM"
    cases = {"none": ([], False), "started": ([start], True), "ended": ([start, end], False), "signalled": ([start, signal_line], False),
             "restarted": ([start, end, start], True)}
    got = {name: rows_orchestrator_running(lines) for name, (lines, _) in cases.items()}
    return all(got[name] == expected for name, (_, expected) in cases.items()), f"{got}"


def cell_h23_collision(scratch: str) -> tuple[bool, str]:
    repo, config, revision_8_config = fake_world(scratch)
    os.makedirs(os.path.join(repo, "research/results"), exist_ok=True)
    clean = check_h23(repo, config, revision_8_config, "start")
    old_dumps = json.loads(json.dumps(config))
    old_dumps["H23"]["dumps_root"] = REVISION_6_SEGMENT_TWO_DUMPS + "-r2"
    inside_segment_one = json.loads(json.dumps(config))
    inside_segment_one["H23"]["dumps_root"] = segment_one_of(revision_8_config).dumps + "/segment-two"
    judge.write_text(os.path.join(repo, segment_two_prefix(FAKE_RUN_DATE) + "-rows-driver-r8.out"), "早先一份\n")
    existing = check_h23(repo, config, revision_8_config, "start")
    outcomes = [check_h23(repo, item, revision_8_config, "start").holds for item in (old_dumps, inside_segment_one)] + [existing.holds]
    return clean.holds and outcomes == [False, False, False], f"干净的过 {clean.holds}；修订 6 第二段导出根、修订 8 第一段导出根里面、已有同名产物：{outcomes}（要都 False）"


def cell_h24_names(scratch: str) -> tuple[bool, str]:
    _, config, revision_8_config = fake_world(scratch)
    good = check_h24(scratch, config, revision_8_config, "start")
    variants = []
    for names, label in ((REVISION_6_SEGMENT_TWO_PEER_NAMES, "e171s2r8"), (tuple(revision_8_config["H18"].values()) + ("research/e171-s2-final-launch",), "e171s2r8"),
                         (tuple(config["H24"][key] for key in PEER_NAME_KEYS), "e171s2")):
        variant = json.loads(json.dumps(config))
        variant["H24"] = dict(zip(PEER_NAME_KEYS, names), label_prefix=label)
        variants.append(check_h24(scratch, variant, revision_8_config, "start").holds)
    return good.holds and variants == [False, False, False], f"模板的落点过 {good.holds}；修订 6 第二段的名字、修订 8 第一段的名字、标签前缀 e171s2：{variants}（要都 False）"


def cell_h25_files(scratch: str) -> tuple[bool, str]:
    repo = os.path.abspath(os.path.join(HERE, "../../.."))
    files = {path: judge.sha256_or_missing(judge.resolve(repo, path)) for path in H25_REQUIRED_FILES}
    good = check_h25(repo, {"H25": {"files": files}}, {}, "start")
    files[H25_REQUIRED_FILES[0]] = "0" * 64
    bad = check_h25(repo, {"H25": {"files": files}}, {}, "start")
    return good.holds and not bad.holds, f"现算的 sha256 登进去过 {good.holds}；改一个 {bad.holds}（要 False）"


def cell_h28_files(scratch: str) -> tuple[bool, str]:
    paths = {key: judge.write_text(os.path.join(scratch, f"{key}.out"), f"{key}\n") for key in H28_FILE_KEYS}
    os.makedirs(os.path.join(scratch, "rows"))
    h28 = dict(paths, rows_root=os.path.join(scratch, "rows"), product_prefix="p-", product_suffix="-r6",
               sha256={path: judge.sha256_of_file(path) for path in paths.values()})
    good = check_h28(scratch, {"H28": h28}, {}, "contrast")
    judge.write_text(paths["comparator_product"], "记下 sha256 之后又改了\n")
    bad = check_h28(scratch, {"H28": h28}, {}, "contrast")
    return good.holds and not bad.holds, f"记下的 sha256 对得上过 {good.holds}；比对器产物记完又改 {bad.holds}（要 False）"


def cell_dry_run_refused(scratch: str) -> tuple[bool, str]:
    dry = {"dry_run": True, "dry_run_note": "自证"}
    start, compare = check_dry_run(scratch, dry, {}, "start"), check_dry_run(scratch, dry, {}, "compare")
    of_revision_8 = check_dry_run(scratch, {}, dry, "start")
    return not start.holds and compare.holds and not of_revision_8.holds, f"干跑配置 start {start.holds}（要 False）、compare {compare.holds}；修订 8 那一份是干跑的 start {of_revision_8.holds}"


def cell_compare_command_inputs(scratch: str) -> tuple[bool, str]:
    counts: dict[str, int] = {}
    extra = (green_line("step-7", [], counts, tag="-attempt2"), green_line("step-9", [], counts))
    repo, config, revision_8_config = fake_world(scratch, extra_lines=extra)
    command = compare_command(repo, config, revision_8_config)
    text = " ".join(command)
    dumps = segment_one_of(revision_8_config).dumps
    good = (f"--baseline {dumps}/{STEP_1_STEM} " in text and f"--stability-baseline {dumps}/baseline-run1-attempt2 " in text
            and f"--stability-baseline {dumps}/baseline-run2 " in text and "--product-suffix=-r8.out" in text
            and text.endswith(f"{segment_two_prefix(FAKE_RUN_DATE)}-comparator-r8.out") and "--mutations-table scripts/experiments/target/fake-tree/crates/mutations.tsv" in text)
    return good, text[-160:]


SELFTEST_CELLS = {"template-shape": cell_template_shape, "unfilled-named": cell_unfilled_named, "gate-research-mutations": cell_gate_research_mutations,
                  "gate-line-counts": cell_gate_line_counts, "t2-covers-the-streams": cell_t2_covers_the_streams,
                  "t2-named-by-the-green-line": cell_t2_named_by_the_green_line, "local-gate": cell_local_gate,
                  "rows-orchestrator-running": cell_rows_orchestrator_running, "h23-collision": cell_h23_collision, "h24-names": cell_h24_names,
                  "h25-files": cell_h25_files, "h28-files": cell_h28_files, "dry-run-refused": cell_dry_run_refused,
                  "compare-command-inputs": cell_compare_command_inputs}
SELFTEST_SWITCHES = {"unfilled-passes": "unfilled-named", "gate-ignores-research-mutations": "gate-research-mutations",
                     "gate-ignores-line-counts": "gate-line-counts", "t2-cover-skipped": "t2-covers-the-streams",
                     "t2-any-product": "t2-named-by-the-green-line", "local-gate-without-rows-start": "local-gate",
                     "running-ignores-end-lines": "rows-orchestrator-running", "h23-collision-skipped": "h23-collision",
                     "h24-old-names-accepted": "h24-names", "h25-sha-skipped": "h25-files", "h28-sha-skipped": "h28-files",
                     "dry-run-accepted": "dry-run-refused"}


def selftest(argument_list: list[str]) -> int:
    os.environ[SELFTEST_VARIABLE] = "1"
    return harness.run_selftest(os.path.abspath(__file__), argument_list, SELFTEST_CELLS, SELFTEST_SWITCHES, BREAK_VARIABLE, SELFTEST_VARIABLE)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
