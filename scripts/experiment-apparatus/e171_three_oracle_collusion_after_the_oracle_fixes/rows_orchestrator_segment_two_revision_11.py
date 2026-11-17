#!/usr/bin/env python3
# task-server-shards: run
"""E171 第 1 次重跑·修订 11：修订 8 那一趟第二段整表的多机编排，带复用（登记 research/prompts/e171-r1-prereg.md 修订 11 第 11.2–11.5、11.9、11.10 节；
复用那一层照 reuse_library.py 文件头「按项判定」）。由 run_segment_two_revision_11_multi_host.sh 起，在仓根跑；修订 11 占位（revision_11_placeholders.json 那一形）
点名修订 10 那一份占位（revision_10_config）。

为什么另写一层：rows_orchestrator_segment_two_revision_10.py（57a7d2ca…）一个字节不改。这一层按路径载入它（开跑先核 sha256，不是就退 3；它照旧按 sha256
载入 rows_orchestrator_segment_two.py 2b91b18a… 与第一段编排器 2ab19c53…），修订 10 那一层的几处（树与 T₂、H8 核对、落点、起跑闸、本机闸、停派文件、让路、
输入清单、S17 联动）由它自己的 run_layer 原样装上；这一层只在第二段编排器上再换这几处：
  ① 整表派发之前（phase_rows）：281 行（编排器现算的行集）照 reuse_library 的请求写法（L 取 T₂ 那份计时的 process_wall）一份计划（ItemRun）：算键、查库、
    写标记（<H23 导出根>/reuse-markers/segment-two-rows/）；取得到的行数 M ≥ 1 时照 RC3 挑 k = max(3, ⌈0.02 × M⌉) 行照跑、只比不入
    （按 sha256(H23 后缀 ‖ 键) 从小到大，挑中的与挑法记一行 name=reuse_sample，即 H32；续跑照第一次挑的那份），行输出根里这些行的旧行目录先删掉
    （输出根只是这一趟的产物），其余取得到的行把判定库里那一份解进行输出根。跨机对照那一行（编排器现算的那一行）在整表与跨机那一次都是 RC2 的 second-run，
    要补位的行（按名字取不到、锚点不是恰 1 次、碰到模块集合）拒算、照跑，补位协调照编排器原样；
  ② 做没做完（is_done）只认这一趟：从判定库取来的，或这一趟的分片跑完、行目录里有 row-result.tsv 的；合成时这一趟分片没覆盖的行（这一趟取来的）由本机
    一个驱动带 --report-only 照取来的行目录重报（run_carry_two），有一行不是这一趟取来的就停；
  ③ 一个分片结束（finish_shard_two）：编排器原来那样收尾，再把这一分片照跑完的行照入库条件判绿或判红（RC1、RC2、RC3 照 reuse_library 比）；
    别的台跑的行先核三样（reuse_library.peer_row_problems），不过判红；S18 触发（那一把键已撤成红）这一分片记没跑成（编排器不再派新行，在跑的跑完再停，退 1）；
    一步的计划收尾（finish）判红同样停（退 1）；
  ④ 跨机对照（phase_cross_host）之前给那一行的跨机那一份一份计划（second-run，照跑）；
  ⑤ 超时重跑（phase_timeout_reruns，只另出产物）：参数带 --timed-out-before（第二段编排器 RERUN_STEP 自己带：调用方显式给，限时再乘 3），
    超时的行照同一套一份计划、取用。
  收尾由标记现合复用账本 <H23 前缀>-reuse-ledger-rows<H23 后缀>.out（登记的行：整表各行、多机时跨机那一份、超时重跑的各行），N + M 不等于登记数退 1。
用法：
  rows_orchestrator_segment_two_revision_11.py run  --config <修订 11 占位> [--repo R]
  rows_orchestrator_segment_two_revision_11.py plan --config <修订 11 占位> [--repo R]   修订 10 的 plan 原样，先打每行拒算的缘由（只读）
  rows_orchestrator_segment_two_revision_11.py --selftest [--cells 格,…] [--rounds N]
退出码：同修订 10 第二段编排层（0、1、2、3、78、143）；账本 N + M 不等于登记数退 1；H31 不过退 78；prune-server 连不上退 75
（判定库只在服务里，reuse_library.py 文件头「按项判定」）；自证只在临时 prune-server 上跑（不是临时服务退 2）。
经 task-server：文件头第二行声明了 run 这一种分片，服务照计划、各片、合并三步起（rows_orchestrator.py 文件头「经 task-server」），
  计划、合并与代跑的片在读配置、碰库之前就交了退出码；不经服务直接调只用这一台，跨机对照报 not_run。

弄坏开关 SEGMENT_TWO_R11_BREAK=<项>（逗号连几个；只在 --selftest 起的子进程里认，SEGMENT_TWO_R11_SELFTEST=1）。逐项写开关名、弄坏哪一处，下一行起写打开后红的格（自证打「✗ 自证 <格名>：…」）：
  SEGMENT_TWO_R11_BREAK=sample-skipped  取得到的行一行都不挑出来照跑（RC3 不做）
      「✗ 自证 reuse-e2e：」
  SEGMENT_TWO_R11_BREAK=peer-checks-skipped  别的台跑的行入库之前不核那三样
      「✗ 自证 peer-checks：」
  SEGMENT_TWO_R11_BREAK=s18-ignored  S18 触发了照样跑完
      「✗ 自证 s18-stops：」
  SEGMENT_TWO_R11_BREAK=stale-rows-done  行输出根里旧的行目录不清，照「有 row-result.tsv 就是做完了」算
      「✗ 自证 stale-rows-redone：」
  SEGMENT_TWO_R11_BREAK=sweep-skipped  每一格之后与末尾不收进程（自证外壳 revision_8_selftest_harness.py 的开关）
      「✗ 自证 no-leftover-processes：」
  SEGMENT_TWO_R11_BREAK=temporary-kept  末尾不删临时目录（自证外壳 revision_8_selftest_harness.py 的开关）
      「✗ 自证 temporary-directory-removed：」
另认 reuse_library.py 的 REUSE_BREAK 与 long_run_item_verdicts.py 的 LONG_RUN_ITEMS_BREAK（各自文件头）。
"""
from __future__ import annotations

import dataclasses
import json
import os
import shutil
import subprocess
import sys
import types

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import device_timing  # noqa: E402
import frozen_tree_lines_revision_8 as judge  # noqa: E402
import reuse_library as reuse  # noqa: E402
import revision_8_selftest_harness as harness  # noqa: E402

BREAK_VARIABLE = "SEGMENT_TWO_R11_BREAK"
SELFTEST_VARIABLE = "SEGMENT_TWO_R11_SELFTEST"
SEGMENT_TWO_R10_PATH = os.path.join(HERE, "rows_orchestrator_segment_two_revision_10.py")
SEGMENT_TWO_R10_SHA256 = "b7e142b6e9a4b6649f6e6a9410127e83e76e84c104aef13736a60b31e76c786f"
EXIT_WRONG_PINNED_FILE = 3
LEDGER_STEM = "reuse-ledger-rows"
MARKERS_NAME = "segment-two-rows"
SCRIPT_KEY = f"{reuse.RESEARCH_RELATIVE}/rows_orchestrator_segment_two_revision_11.py"   # 按项判定里认的脚本名（复用哨兵、复用三问的规范输出）


SEGMENT_TWO_R10_LOADS_SEGMENT_TWO = 'load_pinned(SEGMENT_TWO_ORCHESTRATOR_PATH, SEGMENT_TWO_ORCHESTRATOR_SHA256, "rows_orchestrator_segment_two_7d7b458b")'
try:
    # rows_orchestrator_segment_two.py 与修订 10 一个字节不改（先核各自的 sha256，只差登记过的搬迁也认，reuse_library 的 pinned_sha256_now）：
    # 第二段编排器先按第一段编排器此刻该认的 sha256 载入（reuse_library 的 orchestrator_repin），修订 10 载入时它那一处载第二段编排器的调用换成已经载好的这一份
    reuse.pinned_module("rows_orchestrator_segment_two_7d7b458b")
    layer10 = reuse.load_repinned(SEGMENT_TWO_R10_PATH, SEGMENT_TWO_R10_SHA256, "rows_orchestrator_segment_two_revision_10_627e48c6",
                                  ((SEGMENT_TWO_R10_LOADS_SEGMENT_TWO, 'sys.modules["rows_orchestrator_segment_two_7d7b458b"]'),))
except RuntimeError as wrong_file:
    print(f"✗ {wrong_file}（这一层调的是钉住的那一份的函数；改过就先核这一层，再改钉的 sha256）", file=sys.stderr)
    sys.exit(EXIT_WRONG_PINNED_FILE)
two = layer10.two
one = two.one


def broken(item: str) -> bool:
    return harness.broken(BREAK_VARIABLE, item)


@dataclasses.dataclass
class SegmentTwoReuse:
    """这一层的复用状态。decisions、runs：(步, 行) → 决定、那一份计划（步是 rows、cross、rerun）；fetched：这一趟从判定库取来、放进行输出根的 (步, 行)；
    finished：这一趟的分片跑完的 (步, 行)；registered：账本登记的行名。"""
    repo: str
    markers: str
    state_parent: str
    suffix: str
    decisions: dict
    runs: dict
    fetched: set
    finished: set
    registered: list
    facts: reuse.TreeFacts | None = None


STATE: dict = {"reuse": None}
ORIGINAL: dict = {}


def tree_facts(context) -> reuse.TreeFacts:
    """算请求用的那棵树（第一次用到时现算）。"""
    state = STATE["reuse"]
    if state.facts is None:
        state.facts = reuse.tree_facts(context.repo, context.tree, dict(os.environ))
    return state.facts


def row_limit(context, rerun: bool) -> int | None:
    """L：T₂ 那份计时（编排层给编排器的 --baseline-timing）的 process_wall，max(1800, ⌈3T⌉)，超时重跑再乘 3。"""
    path = os.path.join(context.repo, context.baseline_timing)
    if not os.path.exists(path):
        return None
    baseline = device_timing.read_baseline_timing(open(path, encoding="utf-8").read().splitlines())
    return device_timing.row_time_limit_seconds(baseline.process_wall_seconds, rerun) if baseline.process_wall_seconds is not None else None


def earlier_sample(markers: str) -> list[str] | None:
    """这一趟早先（续跑之前）挑好的 RC3 那几行：续跑照用，不重挑。"""
    lines = [line for line in reuse.event_lines(markers) if " name=reuse_sample " in f" {line} "]
    if not lines:
        return None
    rows = lines[0].split(" rows=", 1)[1].split()[0]
    return [] if rows == "-" else rows.split(",")


def plan_and_fetch(context, step: str, rows: tuple[int, ...], sample: bool, clear_stale: bool = True) -> reuse.ItemRun:
    """派发之前（文件头 ①、④、⑤）：这一步的行一份计划；整表那一次照 RC3 挑 k 行改成照跑、只比不入；取得到的其余行解进行输出根。
    clear_stale：没取来的行的旧行目录也删掉（超时重跑与整表共用行目录，那一步不删，驱动照跑时自己删）。弄坏开关 stale-rows-done 下旧行目录不删。"""
    state = STATE["reuse"]
    facts, selected = tree_facts(context), reuse.selected_row_lines(context.repo)
    units = reuse.segment_two_row_units(rows, context.cross_host_row, (step,))
    requests = {row.name: reuse.row_request(facts, row, selected, row_limit(context, row.rerun), []) for row in units}
    run = reuse.ItemRun(context.repo, SCRIPT_KEY, list(requests.values()), state.state_parent, state.markers, printer=lambda line: one.say(context, line))
    decisions = run.plan()
    if sample and not broken("sample-skipped"):
        chosen = earlier_sample(state.markers)
        if chosen is None:
            chosen = reuse.rows_sample(list(decisions.values()), state.suffix)
            if chosen:
                reuse.append_event(state.markers, f"E7RESULT name=reuse_sample k={len(chosen)} rows={reuse.value(','.join(chosen))} "
                                                  f"taken_before_sampling={sum(1 for decision in decisions.values() if decision.state == reuse.STATE_REUSED)} "
                                                  f"suffix={state.suffix}")
        for unit in chosen:
            if unit in decisions and decisions[unit].state == reuse.STATE_REUSED:
                run.mark_sample(unit)
        decisions = dict(run.decisions)
    step_root = context.step_root(two.step_of(context, step))
    for row in units:
        decision = decisions[row.name]
        state.decisions[(step, row.number)], state.runs[(step, row.number)] = decision, run
        if row.name not in state.registered:
            state.registered.append(row.name)
        row_directory = os.path.join(step_root, f"row-{row.number}")
        if decision.state == reuse.STATE_REUSED or (clear_stale and not broken("stale-rows-done")):
            shutil.rmtree(row_directory, ignore_errors=True)
        if decision.state == reuse.STATE_REUSED:
            run.fetch(row.name, {"row": row_directory})
            state.fetched.add((step, row.number))
    counts = {name: sum(1 for decision in decisions.values() if decision.state == name)
              for name in (reuse.STATE_REUSED, reuse.STATE_FRESH, reuse.STATE_FRESH_CHECK, reuse.STATE_FRESH_ALWAYS)}
    one.say(context, f"  复用（{step}）：{len(units)} 行，库里取来 {counts[reuse.STATE_REUSED]} 行，RC3 抽出来照跑 {counts[reuse.STATE_FRESH_CHECK]} 行，"
                     f"现判 {counts[reuse.STATE_FRESH]} 行，拒算照跑 {counts[reuse.STATE_FRESH_ALWAYS]} 行")
    return run


def run_phase(run: reuse.ItemRun | None, phase, *arguments) -> None:
    """编排器原来的一步；之后这一份计划收尾（文件头 ③）。那一步抛出来时收尾照做、照原样抛；收尾判红这一步停（退 1）。"""
    try:
        phase(*arguments)
    except BaseException:
        if run is not None:
            run.finish()
        raise
    if run is not None and run.finish() != 0:
        raise one.Stopped("按项判定收尾判红（复用哨兵或键在这一趟中途变了，见上面 long_run_item_verdicts 打的那几行）：交主 agent")


def phase_rows_with_reuse(context, hosts, slots, setup, expected_hashes) -> None:
    run = plan_and_fetch(context, "rows", context.rows, True) if not os.path.exists(context.product(two.ROWS_STEP.stem)) else None
    run_phase(run, ORIGINAL["phase_rows"], context, hosts, slots, setup, expected_hashes)


def phase_cross_host_with_reuse(context, hosts, setup, expected_hashes) -> None:
    run = None
    if len(hosts) > 1 and not os.path.exists(context.product(two.cross_host_step(context.cross_host_row).stem)):
        run = plan_and_fetch(context, "cross", (context.cross_host_row,), False)
    run_phase(run, ORIGINAL["phase_cross_host"], context, hosts, setup, expected_hashes)


def phase_timeout_reruns_with_reuse(context, hosts, setup, expected_hashes) -> None:
    timed = one.timed_out_rows(context.product(two.ROWS_STEP.stem))
    run = plan_and_fetch(context, "rerun", timed, False, clear_stale=False) if timed and not os.path.exists(context.product(two.RERUN_STEP.stem)) else None
    run_phase(run, ORIGINAL["phase_timeout_reruns"], context, hosts, setup, expected_hashes)


def is_done_by_the_library(context, step, row: int) -> bool:
    """这一步这一行这一趟做完了（文件头 ②）：从判定库取来的，或这一趟的分片跑完、行目录里有 row-result.tsv 的。弄坏开关 stale-rows-done 下照编排器原来的判法。"""
    if broken("stale-rows-done"):
        return ORIGINAL["is_done"](context, step, row)
    state = STATE["reuse"]
    if (step.key, row) in state.fetched:
        return True
    return (step.key, row) in state.finished and os.path.exists(os.path.join(context.step_root(step), f"row-{row}", "row-result.tsv"))


def run_carry_from_the_library(context, step, rows: tuple[int, ...], expected_hashes: dict[str, str]) -> dict:
    """合成时这一趟分片没覆盖的行（文件头 ②）：本机一个驱动带 --report-only 照这一趟取来的行目录重报。有一行不是这一趟取来的就停。"""
    state = STATE["reuse"]
    missing = [row for row in rows if (step.key, row) not in state.fetched]
    if missing:
        raise one.Stopped(f"{step.key}：第 {one.rows_text(missing)} 行既不是这一趟跑的、也不是这一趟从判定库取来的（输出根里的旧行不重报）")
    work_root = os.path.join(context.attempt_dir, f"carry-work-{step.key}")
    reporting = dataclasses.replace(step, flags=step.flags + ("--report-only",))
    process, record = two.launch_local_two(context, expected_hashes, f"{step.key}-carry", reporting, rows, one.registered_host_threads(), work_root, "carry", set())
    code = process.wait()
    if one.STOP.is_set():
        raise one.Interrupted()
    if not two.finish_shard_two(context, record, code):
        raise one.Stopped(f"{step.key}：本机照取来的行目录重报第 {one.rows_text(rows)} 行那个驱动没跑成（退 {code}）")
    return record


def frozen_snapshot_digest(context) -> str:
    if "frozen_snapshot_digest" not in STATE:
        STATE["frozen_snapshot_digest"] = one.row_driver_module(context).snapshot_digest(os.path.join(context.shards, "frozen-snapshot"))[0]
    return STATE["frozen_snapshot_digest"]


def peer_check_problems(context, record: dict, row_directory: str, decision: reuse.Decision) -> list[str]:
    if broken("peer-checks-skipped"):
        return []
    product = open(record["product"], encoding="utf-8").read() if os.path.exists(record["product"]) else ""
    return reuse.peer_row_problems(row_directory, decision, product, frozen_snapshot_digest(context))


def finish_shard_with_reuse(context, record: dict, code: int) -> bool:
    """记下这一分片跑完的行（文件头 ②），编排器原来的 finish_shard_two，再把这一分片照跑完的行照入库条件判绿或判红（别的台的先核三样）；
    S18 触发这一分片记没跑成。"""
    state = STATE["reuse"]
    if record["kind"] in ("local", "peer"):
        state.finished.update((record["step"], row) for row in record["rows"])
    shard_ran_through = ORIGINAL["finish_shard_two"](context, record, code)
    if record["kind"] not in ("local", "peer"):
        return shard_ran_through
    step = two.step_of(context, record["step"])
    for row in record["rows"]:
        decision, run = state.decisions.get((record["step"], row)), state.runs.get((record["step"], row))
        row_directory = os.path.join(context.step_root(step), f"row-{row}")
        if decision is None or run is None or not os.path.exists(os.path.join(row_directory, "row-result.tsv")):
            continue
        problems = peer_check_problems(context, record, row_directory, decision) if record["kind"] == "peer" else []
        insertable = (False, "peer-check-" + ",".join(problems)) if problems else reuse.row_insertable(row_directory)
        device_exit = reuse.row_result_fields(row_directory).get("device_exit", "-")
        outcome = run.record(decision.unit, {"row": row_directory}, int(device_exit) if device_exit.lstrip("-").isdigit() else -1, insertable)
        one.say(context, f"  入库 {decision.unit}（{one.host_name(record['host'])}）：{outcome}（{insertable[1] or '入库条件过'}）")
    problems = reuse.s18_problems(state.markers)
    if problems and not broken("s18-ignored"):
        one.say(context, f"✗ S18 触发（{len(problems)} 行事件，第一行：{problems[0][:200]}）：那一把键已撤成红，这一分片记没跑成，交主 agent")
        return False
    return shard_ran_through


def plan_lines(context) -> list[str]:
    """plan：整表每行的键与库里有没有判绿的、拒算的缘由（只读：不写标记、不写库）。"""
    state = STATE["reuse"]
    lines = [f"── 复用（修订 11）── 判定库 prune-server apparatus-items/{reuse.KIND}；标记 {state.markers}；RC3 挑法 sha256({state.suffix} ‖ 键)"]
    facts, selected, limit = tree_facts(context), reuse.selected_row_lines(context.repo), row_limit(context, False)
    requests = [reuse.row_request(facts, row, selected, limit, []) for row in reuse.segment_two_row_units(context.rows, context.cross_host_row)]
    keys = reuse.keyed_items(context.repo, requests)
    counts: dict[str, int] = {}
    for request in requests:
        key, why = keys[request.unit]
        if request.reason or key is None:
            verdict = f"拒算（{request.reason or 'refused'}）"
            lines.append(f"行 {request.unit}：{verdict}{'' if request.reason else '：' + why}")
        else:
            verdict = "库里有判绿的" if reuse.stored_value(context.repo, key) is not None else "库里没有判绿的"
            lines.append(f"行 {request.unit}：键 {key[:16]}；{verdict}")
        counts[verdict] = counts.get(verdict, 0) + 1
    return lines + ["各类行数：" + "；".join(f"{verdict} {number}" for verdict, number in sorted(counts.items()))]


def run_two_with_ledger(context, plan_only: bool) -> int:
    """编排器原来的 run_two；收尾现合复用账本（跑完装进 research/results，没跑完只写日志目录）。plan 另打每行的键。"""
    if plan_only:
        code = ORIGINAL["run_two"](context, True)
        for line in plan_lines(context):
            print(line)
        return code
    state = STATE["reuse"]
    code = None
    try:
        code = ORIGINAL["run_two"](context, False)
    finally:
        text, covers = reuse.ledger_text(state.markers, state.registered)
        os.makedirs(context.logs, exist_ok=True)
        with open(os.path.join(context.logs, f"{LEDGER_STEM}.out"), "w", encoding="utf-8") as handle:
            handle.write(text)
    if code == 0:
        installed = reuse.install_ledger(context.product(LEDGER_STEM), text)
        one.say(context, f"  复用账本 {os.path.relpath(installed, context.repo)}：{text.strip().splitlines()[-1]}")
        if not covers:
            one.say(context, "✗ 复用账本 N + M 不等于登记的行数（第 11.4 节判红）")
            return 1
    return code


def install(repo: str, segment_two, plan: bool) -> None:
    state_parent = os.path.join(repo, segment_two.dumps, "reuse-items", MARKERS_NAME)
    if not plan:
        shutil.rmtree(state_parent, ignore_errors=True)   # 状态目录只属于一趟：上一趟停在中途留下的不读、删掉
    STATE["reuse"] = SegmentTwoReuse(repo, os.path.join(repo, segment_two.dumps, "reuse-markers", MARKERS_NAME), state_parent, segment_two.suffix,
                                     {}, {}, set(), set(), [])
    ORIGINAL.update({"phase_rows": two.phase_rows, "finish_shard_two": two.finish_shard_two, "phase_cross_host": two.phase_cross_host,  # 换掉之前逐个点名留下原件
                     "phase_timeout_reruns": two.phase_timeout_reruns, "run_two": two.run_two, "is_done": one.is_done, "run_carry_two": two.run_carry_two})
    two.phase_rows = phase_rows_with_reuse
    two.finish_shard_two = finish_shard_with_reuse
    two.phase_cross_host = phase_cross_host_with_reuse
    two.phase_timeout_reruns = phase_timeout_reruns_with_reuse
    two.run_two = run_two_with_ledger
    two.run_carry_two = run_carry_from_the_library
    one.is_done = is_done_by_the_library


def main(argument_list: list[str]) -> int:
    for variable, selftest_variable in ((BREAK_VARIABLE, SELFTEST_VARIABLE), (reuse.BREAK_VARIABLE, reuse.SELFTEST_VARIABLE),
                                        (layer10.BREAK_VARIABLE, layer10.SELFTEST_VARIABLE), (two.BREAK_VARIABLE, two.SELFTEST_VARIABLE),
                                        ("ROWS_ORCHESTRATOR_BREAK", "ROWS_ORCHESTRATOR_SELFTEST")):
        refusal = harness.refuse_break_outside_selftest(variable, selftest_variable)
        if refusal:
            print(refusal, file=sys.stderr)
            return 2
    if os.environ.get(two.SELFTEST_ROWS_VARIABLE) and os.environ.get(two.SELFTEST_VARIABLE) != "1":
        print(f"✗ {two.SELFTEST_ROWS_VARIABLE} 只在自证起的子进程里认（平常跑整表）；设了它就拒（退 2）", file=sys.stderr)
        print('     → 怎么办：平常跑时 unset 那个变量再起；它只给 --selftest 起的子进程用', file=sys.stderr)
        return 2
    if argument_list[:1] == ["--selftest"]:
        return selftest(argument_list[1:])
    import argparse  # noqa: E402
    parser = argparse.ArgumentParser(description="E171 修订 11：修订 8 那一趟第二段整表的多机编排，带复用")
    parser.add_argument("mode", choices=("run", "plan"))
    parser.add_argument("--config", required=True, help="修订 11 占位（revision_11_placeholders.json 那一形）")
    parser.add_argument("--repo", default=os.getcwd())
    arguments = parser.parse_args(argument_list)
    repo = os.path.abspath(arguments.repo)
    if arguments.mode == "run":
        phase_exit = one.shard_phase_before_layer(repo, say=lambda line: print(line, flush=True))
        if phase_exit is not None:
            return phase_exit
    try:
        reuse_config = json.load(open(arguments.config, encoding="utf-8"))
        revision_10_path = judge.resolve(repo, reuse_config["revision_10_config"])
        config, revision_8_config = layer10.placeholders.load_configs(repo, revision_10_path)
        segment_two = layer10.placeholders.segment_two_of(config)
        if arguments.mode == "run":
            if reuse_config.get("dry_run"):
                raise reuse.RunCondition(f"修订 11 占位是干跑配置（{reuse_config.get('dry_run_note') or '没写说明'}），只给 plan 用")
            problems = reuse.h31_problems(repo, reuse_config)
            if problems:
                raise reuse.RunCondition("H31 不过：" + "；".join(problems[:4]))
            reuse.check_service_reachable(repo)
        else:
            print(f"修订 11 占位 {os.path.abspath(arguments.config)}（dry_run={str(bool(reuse_config.get('dry_run'))).lower()}；{reuse_config.get('dry_run_note') or '-'}）；"
                  f"H31 {'；'.join(reuse.h31_problems(repo, reuse_config)) or '与现在的文件逐个相同'}；修订 10 占位 {revision_10_path}")
    except (reuse.RunCondition, judge.RunConditionError, KeyError, OSError) as error:
        print(f"✗ 运行条件不满足（退 78）：{error}", flush=True)
        print('     → 怎么办：照上面那句补齐运行条件（配置、冻结树、设备文件）再用同一条命令起', flush=True)
        return one.EXIT_RUN_CONDITION
    install(repo, segment_two, arguments.mode == "plan")
    if arguments.mode == "plan":
        for line in static_plan_lines(repo, judge.tree_of(revision_8_config)):
            print(line)
    return layer10.run_layer(repo, revision_10_path, arguments.mode == "plan")


def static_plan_lines(repo: str, tree: str) -> list[str]:
    """plan 在交编排层之前先打的那几行（编排层在 T₂ 取不到时不起 run_two）：整表每行拒算与否（与 L 无关），键等 T₂ 定了现算。"""
    try:
        facts = reuse.tree_facts(repo, tree, dict(os.environ))
    except OSError as error:
        return [f"✗ 算请求用的树 {tree} 读不到：{error}"]
    selected, reasons = reuse.selected_row_lines(repo), {}
    for row in reuse.segment_two_row_units(tuple(sorted(selected)), None):  # 整表行集：两张行表第一列之并（E171 第 2 次重跑登记读法 4，不写死）
        reason = reuse.row_request(facts, row, selected, None, []).reason or "照常查库"
        reasons.setdefault(reason, []).append(row.number)
    return [f"复用·树 {tree}：装置入口 {facts.device_entry or '找不到'}；整表 {len(selected)} 行里 " +
            "；".join(f"{reason} {len(rows)} 行" + (f"（第 {','.join(map(str, rows[:8]))} 行）" if reason != "照常查库" else "") for reason, rows in sorted(reasons.items())) +
            "；各行的键要 L，T₂ 定了之后由编排层现算（跨机对照那一行另记 second-run）"]


# ── 自证：借第二段编排器的假世界（假驱动、假的代跑那一片、假机器清单与资源），假仓与假树补上算键要的几样（reuse_library.furnish_rows_world；
#    装置入口经 LONG_RUN_ITEMS_SELFTEST_ENTRIES 换成假入口），配置照修订 10 自证那一形逐遍另写 ──
FAKE_DRIVER = os.path.join(HERE, "fake_row_driver.py")
FAKE_ROWS = tuple(range(1, 13))


@dataclasses.dataclass
class WorldEleven:
    root: str
    base: object
    environment: dict


def fake_world_eleven(root: str, hosts_code: int = 3) -> WorldEleven:
    base = two.fake_world_two(os.path.join(root, "world"), rows=",".join(map(str, FAKE_ROWS)), hosts_code=hosts_code)
    reuse.furnish_rows_world(base.repo, os.path.join(base.repo, "scripts/experiment-apparatus/target/fake-frozen-tree"), FAKE_ROWS)
    entries = reuse.selftest_entries(os.path.join(root, "entries.tsv"), [name for row in FAKE_ROWS for name in (f"seg2-row-{row}", f"seg2-cross-row-{row}",
                                                                                                             f"seg2-rerun-row-{row}")])
    checks = reuse.items_library.lib_reuse_checks
    environment = dict(base.environment, SEGMENT_TWO_R10_SELFTEST="1", SEGMENT_TWO_R10_SKIP_CONFIG_CHECK="1", SEGMENT_TWO_R10_YIELD_RECHECK_SECONDS="0.2",
                       SEGMENT_TWO_R10_RESEARCH_ROOT=two.REAL_REPOSITORY, SEGMENT_TWO_R11_SELFTEST="1", REUSE_SELFTEST="1", ROWS_ORCHESTRATOR_DRIVER=FAKE_DRIVER,
                       **{reuse.items_library.LIBRARY_DIRECTORY_VARIABLE: os.path.join(root, "library"), reuse.items_library.SUBSTITUTION_VARIABLE: entries,
                          checks.SENTINEL_COUNT_VARIABLE: reuse.ALL_SENTINELS})
    return WorldEleven(root, base, environment)


def pass_config(world: WorldEleven, name: str) -> str:
    """一遍的三份配置（照 rows_orchestrator_segment_two_revision_10.fake_world_ten 那一形；H23 各遍不同，修订 8 那一份与 T₂ 各遍相同）。交回修订 11 那一份。"""
    placeholders = layer10.placeholders
    tree = os.path.join(world.base.repo, "scripts/experiment-apparatus/target/fake-frozen-tree")
    segment_one = placeholders.SegmentOne(tree="scripts/experiment-apparatus/target/fake-frozen-tree", snapshot_digest=judge.row_driver.snapshot_digest(tree)[0],
                                          mutations_sha256=judge.sha256_of_file(os.path.join(tree, judge.MUTATIONS_TABLE)), run_date=layer10.FAKE_RUN_DATE,
                                          suffix="-r8", dumps="scripts/experiment-apparatus/target/fake-segment-one-dumps", logs=os.path.join(world.root, "segment-one-logs"))
    judge.write_text(os.path.join(world.base.repo, segment_one.timing(placeholders.STEP_1_STEM)),
                     "E7TIMING name=stream_time stream=a elapsed_ns=50000000000\nE7TIMING name=process_wall seconds=75.274 exit=0\n")
    judge.write_text(segment_one.sequence_log, "\n".join(layer10.segment_one_lines(segment_one, world.base.repo, "finished")) + "\n")
    revision_8 = {"tree": segment_one.tree, "H3": {"snapshot_digest": segment_one.snapshot_digest}, "H8": {"mutations_table_sha256": segment_one.mutations_sha256},
                  "H16": {"run_date": layer10.FAKE_RUN_DATE, "run_suffix": "-r8", "dumps_root": segment_one.dumps, "logs_directory": segment_one.logs},
                  "H18": placeholders.revision_8.template()["H18"]}
    config = placeholders.template()
    config["revision_8_config"] = judge.write_text(os.path.join(world.root, f"revision-8-{name}.json"), json.dumps(revision_8, ensure_ascii=False))
    config["H23"] = {"run_date": f"2026-10-{name}", "run_suffix": "-r8", "dumps_root": f"scripts/experiment-apparatus/target/e171-r1-dumps/fake-segment-two-{name}",
                     "logs_directory": os.path.join(world.root, f"segment-two-logs-{name}"), "work_root": os.path.join(world.root, f"segment-two-work-{name}")}
    revision_10 = judge.write_text(os.path.join(world.root, f"revision-10-{name}.json"), json.dumps(config, ensure_ascii=False))
    h31 = {path: reuse.sha256_file(path) for path in (os.path.abspath(__file__), os.path.join(HERE, "reuse_library.py"))}
    return judge.write_text(os.path.join(world.root, f"revision-11-{name}.json"), json.dumps({"revision_10_config": revision_10, "H31": {"files": h31}}))


def run_pass(world: WorldEleven, name: str, extra: dict[str, str] | None = None, served: bool = False) -> tuple[int, str]:
    """起这一层一遍；served 时照服务的分片三步起（两台的那一格要别的台的行）。"""
    command = [sys.executable, "-B", os.path.abspath(__file__), "run", "--config", pass_config(world, name), "--repo", world.base.repo]
    if served:
        command = one.served_command(world.base.root, command)
    completed = subprocess.run(command, cwd=world.base.repo, env=dict(world.environment, **(extra or {})), capture_output=True, text=True, timeout=600)
    return completed.returncode, completed.stdout + completed.stderr


def product_of(world: WorldEleven, name: str, stem: str) -> str:
    return os.path.join(world.base.repo, f"{layer10.placeholders.segment_two_prefix(f'2026-10-{name}')}-{stem}-r8.out")


def events_of(world: WorldEleven, name: str) -> list[str]:
    return reuse.event_lines(os.path.join(world.base.repo, f"scripts/experiment-apparatus/target/e171-r1-dumps/fake-segment-two-{name}/reuse-markers", MARKERS_NAME))


def driver_lines(world: WorldEleven, kind: str) -> list[str]:
    """假驱动的 driver.log 里以 kind 起头的那几行：ran（照跑的行）、reported（--report-only 重报的行）。"""
    path = os.path.join(world.base.root, "driver.log")
    return [line.strip() for line in open(path, encoding="utf-8") if line.startswith(kind + " ")] if os.path.exists(path) else []


def merged_rows_without_timing(path: str) -> list[str]:
    return sorted(one.row_line_without_timing(line.rstrip("\n")) for line in open(path, encoding="utf-8") if line.startswith("row ")) if os.path.exists(path) else []


def ledger_last_line(world: WorldEleven, name: str) -> str:
    path = product_of(world, name, LEDGER_STEM)
    return open(path, encoding="utf-8").read().strip().splitlines()[-1] if os.path.exists(path) else "账本不在"


def cell_reuse_end_to_end(scratch: str) -> tuple[bool, str]:
    """只用本机：第一遍库空（12 行全部照跑、判绿入库）；第二遍：跨机对照那一行（第 1 行，RC2 的 second-run）照跑，取来的 11 行里 RC3 挑 3 行照跑、与库里那一份
    比都相同，其余 8 行取来不派、由带 --report-only 的驱动重报；合成产物去耗时列与第一遍逐行相同；账本 reused=8、sample_size=3、check_mismatches=0。"""
    world = fake_world_eleven(scratch)
    first_code, _ = run_pass(world, "11")
    first_ran, first_reported = len(driver_lines(world, "ran")), len(driver_lines(world, "reported"))
    second_code, output = run_pass(world, "12")
    second_ran = sorted(int(one.key_values(line)["row"]) for line in driver_lines(world, "ran")[first_ran:])
    second_reported = len(driver_lines(world, "reported")) - first_reported
    events = events_of(world, "12")
    sample_line = [line for line in events if " name=reuse_sample " in f" {line} "]
    sampled = sorted(int(name.rsplit("-", 1)[1]) for name in sample_line[0].split(" rows=", 1)[1].split()[0].split(",")) if sample_line else []
    sample_checks = [line for line in events if " name=reuse_check " in f" {line} " and " kind=sample " in f" {line} "]
    ledger = ledger_last_line(world, "12")
    same = merged_rows_without_timing(product_of(world, "11", "rows-driver")) == merged_rows_without_timing(product_of(world, "12", "rows-driver")) != []
    good = (first_code == 0 and first_ran == 12 and second_code == 0 and len(sampled) == 3 and second_ran == sorted([1] + sampled) and second_reported == 8
            and len(sample_checks) == 3 and all(" identical=true " in f" {line} " for line in sample_checks) and same
            and " reused=8 " in ledger and " sample_size=3 " in ledger and " check_mismatches=0 " in ledger and " units_cover_registered=true " in ledger)
    return good, (f"第一遍退 {first_code}、照跑 {first_ran} 行；第二遍退 {second_code}、照跑第 {second_ran} 行（RC3 挑的 {sampled}）、重报 {second_reported} 行；"
                  f"抽样比 {len(sample_checks)} 次；产物去耗时列相同 {same}；账本 {ledger[22:140]}；第二遍末行 {(output.strip().splitlines() or [''])[-1][:120]}")


def cell_stale_rows_redone(scratch: str) -> tuple[bool, str]:
    """整表的行输出根里先放一份旧的第 5 行（有 row-result.tsv、判定库里没有）：它不当做完，照跑（12 行都照跑）。"""
    world = fake_world_eleven(scratch)
    stale = os.path.join(world.base.repo, "scripts/experiment-apparatus/target/e171-r1-dumps/fake-segment-two-51", "segment2-rows", "row-5")
    judge.write_text(os.path.join(stale, "row-result.tsv"), "row_number\t5\nstatus\tok\ndevice_exit\t0\n")
    code, output = run_pass(world, "51")
    ran = sorted(int(one.key_values(line)["row"]) for line in driver_lines(world, "ran"))
    return code == 0 and ran == list(FAKE_ROWS), f"退 {code}；照跑第 {ran} 行；末行 {(output.strip().splitlines() or [''])[-1][:120]}"


def cell_reuse_two_hosts(scratch: str) -> tuple[bool, str]:
    """两台：别的台跑的行入库之前核三样（假驱动的 mutation_line_sha256 不是变异行的 sha256，所以别的台的行都判红、各记一行 peer-check），
    本机的行照入库条件判绿；跨机对照那一份登记、照跑；账本 units=13、fresh_always=2。"""
    world = fake_world_eleven(scratch, hosts_code=0)
    code, output = run_pass(world, "21", served=True)
    events = events_of(world, "21")
    inserts = [line for line in events if " name=reuse_insert " in f" {line} "]
    peer_refused = [line for line in inserts if "reason=peer-check-" in line]
    green = [line for line in inserts if " state=green " in f" {line} "]
    ledger = ledger_last_line(world, "21")
    good = (code == 0 and peer_refused and all("mutation_line_sha256" in line for line in peer_refused) and green
            and len(inserts) == 13 and " units=13 " in ledger and " fresh_always=2 " in ledger)
    return good, f"退 {code}；别的台的行判红（peer-check）{len(peer_refused)} 行；本机判绿 {len(green)} 行；入库事件 {len(inserts)} 行；账本 {ledger[22:120]}"


def cell_peer_checks(scratch: str) -> tuple[bool, str]:
    raw = "m-row-two\tcrates/core/src/other.rs\talpha\tbeta\t-p core\tt_two"
    digest = reuse.sha256_bytes(raw.encode("utf-8"))
    decision = reuse.Decision("seg2-row-5", "driver-row", "seg2-row-5", "k", reuse.STATE_FRESH, "no-entry",
                              facts={"row": 5, "mutation_line_sha256": digest, "limit_seconds": 1800})
    STATE["frozen_snapshot_digest"] = "d" * 64
    outcomes = {}
    for case, row_number, line_digest, limit, snapshot in (("good", "5", digest, "1800", "d" * 64), ("row_number", "6", digest, "1800", "d" * 64),
                                                           ("mutation_line_sha256", "5", "0" * 64, "1800", "d" * 64),
                                                           ("row_time_limit", "5", digest, "5400", "d" * 64), ("snapshot_digest", "5", digest, "1800", "e" * 64)):
        row_directory = os.path.join(scratch, case, "row-5")
        judge.write_text(os.path.join(row_directory, "row-result.tsv"), f"row_number\t{row_number}\nmutation_line_sha256\t{line_digest}\n")
        product = judge.write_text(os.path.join(scratch, case, "product.out"), f"name=row_time_limit seconds={limit} baseline_seconds=1\nsnapshot_digest={snapshot} files=3\n")
        outcomes[case] = peer_check_problems(types.SimpleNamespace(shards=scratch), {"product": product}, row_directory, decision)
    return outcomes["good"] == [] and all(outcomes[case] == [case] for case in outcomes if case != "good"), f"各样：{outcomes}"


def cell_s18_stops(scratch: str) -> tuple[bool, str]:
    """第二遍本机的假驱动往导出里掺一个字：跨机对照那一行（second-run）与 RC3 抽中的行照跑、与库里那一份比不同 → S18：那一把键撤成红、
    那一分片记没跑成、退 1。"""
    world = fake_world_eleven(scratch)
    first_code, _ = run_pass(world, "31")
    second_code, output = run_pass(world, "32", {"FAKE_DRIVER_SALT_HOST": "1", "FAKE_DRIVER_SALT_PART": "dump"})
    mismatched = [line for line in events_of(world, "32") if " name=reuse_check " in f" {line} " and " identical=false " in f" {line} "]
    return first_code == 0 and second_code == 1 and bool(mismatched) and "S18 触发" in output, \
        f"第一遍退 {first_code}；第二遍（导出掺字）退 {second_code}；读数不同 {len(mismatched)} 行"


def cell_reuse_three_questions(scratch: str) -> tuple[bool, str]:
    """复用三问与复用哨兵的六格（reuse_library.run_reuse_cells）：这一层的两行 seg2-row-1、seg2-row-2，规范输出 REUSE_SELFTEST script=<本文件>。"""
    return reuse.run_reuse_cells(scratch, SCRIPT_KEY, ("seg2-row-1", "seg2-row-2"))


def selftest_cell_functions():
    """自证的各格，格名到函数。表写在函数体里：写成模块级赋值，会在 import 这个模块时把全部自证格连同它们起的子进程拖进 import 一方的判法单元闭包。"""
    return {"reuse-e2e": cell_reuse_end_to_end, "stale-rows-redone": cell_stale_rows_redone, "reuse-two-hosts": cell_reuse_two_hosts,
            "peer-checks": cell_peer_checks, "s18-stops": cell_s18_stops, "reuse-three-questions": cell_reuse_three_questions}


SELFTEST_SWITCHES = {"sample-skipped": "reuse-e2e", "peer-checks-skipped": "peer-checks", "s18-ignored": "s18-stops", "stale-rows-done": "stale-rows-redone"}


def selftest(argument_list: list[str]) -> int:
    problem = reuse.prune_server_client.selftest_service_problem()
    if problem:   # 判定库只在 prune-server 里：自证只在临时服务上跑，不写真服务（文件头「退出码」）
        print(f"✗ rows_orchestrator_segment_two_revision_11.py 自证没跑：{problem}", file=sys.stderr)
        print(f"     → 怎么办：{reuse.prune_server_client.SELFTEST_SERVICE_REMEDY}", file=sys.stderr)
        return 2
    os.environ[SELFTEST_VARIABLE] = "1"
    return harness.run_selftest(os.path.abspath(__file__), argument_list, harness.cells_reporting_progress(selftest_cell_functions()), SELFTEST_SWITCHES,
                                BREAK_VARIABLE, SELFTEST_VARIABLE)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
