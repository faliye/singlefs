#!/usr/bin/env python3
"""E171 第 1 次重跑·修订 11：修订 8 那一趟第 0–10 步按单元编排，带复用（登记 research/prompts/e171-r1-prereg.md 修订 11 第 11.4、11.5、11.9、11.10 节
与其后「主 agent 认定：修订 11 交回」；复用那一层照 reuse_library.py 文件头「按项判定」）。由 run_segment_one_revision_11_multi_host.sh 起，在仓根跑。

为什么另写一层：segment_one_units_revision_8.py 与单元脚本 segment_one_unit_revision_8.sh 一个字节不改（第 2 次重跑登记修订 7、8 改了前者的线程分法、
删了两份里的线程包装，之后按现内容重钉了 UNITS_R8_SHA256，现钉的值就是那个常量；修订 10 把两份单元脚本取线程那一处改成取每台默认重活线程数，修订 11 把前者每个单元的线程改成取额度，同样按现内容重钉）。这一层按路径载入前者（开跑先核 sha256，
不是就退 3），单元表、依赖、次序与资源、续跑、一步停、逐行编排的起法都调它原来的函数；改的只有第 11.10 节表那几处：
  ① 单元脚本换成 segment_one_unit_revision_11.sh（修订 8 那份照抄，只在读数进程那几处加取用），逐行编排换成 rows_orchestrator_revision_11.py
    （修订 11 占位的路径经环境变量 E171_REVISION_11_CONFIG 交给它）；
  ② 加单元 reuse-selfproof（RP2，依赖 step-0）；
  ③ 起一个单元之前（unit_environment）：这个单元装的每个读数单元（reuse_library.readings_of_owner）照 reuse_library 的请求写法一份计划（ItemRun）：
    算键、查库、写标记（<H16 导出根>/reuse-markers/units/）；取得到的写进单元目录的 reuse-entries.tsv（步名<TAB>项名），REUSE_LIBRARY 给这一份计划的状态目录，
    交单元脚本取用；拒算的（step-9、step-10-run-2 这类 second-run 与别几类，reuse_library 文件头「按项判定」）照跑。
    上游读数的位置取上游那一单元的标记（它结束时记下的角色路径），第 4 步各行的 L 取 step-1 计时的 process_wall（max(1800, ⌈3T⌉)）；
  ④ 单元结束时（finish_unit）：单元脚本记下的每一步照跑出来的读数照入库条件判绿或判红（判法红不红不影响入库），RC1、RC2 照 reuse_library 比；
    这一份计划收尾（finish）；S18 触发（账本事件里有 identical=false 或 name=reuse_sentinel）那一把键已撤成红，这个单元记红
    （修订 8 的一步停：不再起新单元、写停派文件、逐行编排不再派新行），交主 agent；
  ⑤ 收尾：由标记现合复用账本 <H16 前缀>-reuse-ledger-units<H16 后缀>.out（登记的读数单元：第 0–10 步各单元装的，research-mutations 排了才算），
    N + M 不等于登记数退 1；没跑完时账本只写进日志目录 reuse-ledger-units.out。
做没做完：修订 8 的续跑照它自己的产物判单元（这一层不改）；读数单元可不可以复用只认判定库（这一趟的计划），输出根里的旧产物不当复用来源。
修订 11 占位（revision_11_placeholders.json，路径不是绝对的相对仓根）：revision_8_config、H31（新文件 → sha256：run 之前核，没填或对不上退 78；
  主 agent 认过才起跑）；dry_run=true 的只给 plan。判定库只在 prune-server 那一份（不认库根、不能关查库）。
  判定库连不上退 75。自证只在临时 prune-server 上跑（开跑前调 selftest_service_problem()，不是临时服务退 2、一格都不跑）。
用法：
  segment_one_units_revision_11.py run  --config <修订 11 占位> [--repo R]
  segment_one_units_revision_11.py plan --config <修订 11 占位> [--repo R]   修订 8 的 plan 全部原样，另打每个读数单元的键与库里有没有判绿的（只读，不写标记）
  segment_one_units_revision_11.py --selftest [--cells 格,…] [--rounds N]
退出码：同修订 8 单元编排（0、1、2、78、143）；另 3：segment_one_units_revision_8.py 不是钉的那一份；账本 N + M 不等于登记数退 1。

自证只换假件的变量（只在 UNITS_R11_SELFTEST=1 时认，另带修订 8 的 UNITS_R8_* 与 long_run_item_verdicts 的 LONG_RUN_ITEMS_*）：UNITS_R11_FAKE_* 交给假单元脚本。
弄坏开关 UNITS_R11_BREAK=<项>（逗号连几个；只在 --selftest 起的子进程里认）。逐项写开关名、弄坏哪一处，下一行起写打开后红的格（自证打「✗ 自证 <格名>：…」）：
  UNITS_R11_BREAK=h31-unchecked  不核 H31（没填也起跑）
      「✗ 自证 h31-refused：」
  UNITS_R11_BREAK=s18-ignored  S18 触发了照样记绿
      「✗ 自证 s18-stops：」
  UNITS_R11_BREAK=report-only-dropped  单元脚本 segment_one_unit_revision_11.sh 取来的驱动行起驱动不带 --report-only（开关在单元脚本 driver_step 里）
      「✗ 自证 unit-script-driver-report-only：」
  UNITS_R11_BREAK=sweep-skipped  每一格之后与末尾不收进程（自证外壳 revision_8_selftest_harness.py 的开关）
      「✗ 自证 no-leftover-processes：」
  UNITS_R11_BREAK=temporary-kept  末尾不删临时目录（自证外壳 revision_8_selftest_harness.py 的开关）
      「✗ 自证 temporary-directory-removed：」
另认 reuse_library.py 的 REUSE_BREAK 与 long_run_item_verdicts.py 的 LONG_RUN_ITEMS_BREAK（各自文件头）。
"""
from __future__ import annotations

import argparse
import dataclasses
import importlib.util
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

BREAK_VARIABLE = "UNITS_R11_BREAK"
SELFTEST_VARIABLE = "UNITS_R11_SELFTEST"
CONFIG_VARIABLE = "E171_REVISION_11_CONFIG"
UNITS_R8_PATH = os.path.join(HERE, "segment_one_units_revision_8.py")
UNITS_R8_SHA256 = "e361ef43853753fa97306909e90a4fa933e8190acc7dc19528204720a6c8bb22"
UNIT_SCRIPT = os.path.join(HERE, "segment_one_unit_revision_11.sh")
ROWS_ORCHESTRATOR = os.path.join(HERE, "rows_orchestrator_revision_11.py")
EXIT_WRONG_PINNED_FILE = 3
SELFPROOF_UNIT = "reuse-selfproof"
LEDGER_STEM = "reuse-ledger-units"
SCRIPT_KEY = f"{reuse.RESEARCH_RELATIVE}/segment_one_units_revision_11.py"   # 按项判定里认的脚本名（复用哨兵、复用三问的规范输出）


def load_pinned(path: str, expected: str, module_name: str) -> types.ModuleType:
    """按路径载入（不改它、不写 .pyc）；sha256 不对就拒（只差登记过的搬迁也认：reuse_library 的 pinned_sha256_now）。"""
    if reuse.pinned_sha256_now(path, expected) is None:
        raise RuntimeError(f"{path} 的 sha256 是 {reuse.sha256_file(path)}，不是 {expected}（把登记的搬迁倒着换回去之后也不是）")
    specification = importlib.util.spec_from_file_location(module_name, path)
    module = importlib.util.module_from_spec(specification)
    sys.dont_write_bytecode = True
    sys.modules[specification.name] = module
    specification.loader.exec_module(module)
    return module


try:
    units8 = load_pinned(UNITS_R8_PATH, UNITS_R8_SHA256, "segment_one_units_revision_8_cc52fcfc")
except RuntimeError as wrong_file:
    print(f"✗ {wrong_file}（这一层调的是钉住的那一份的函数；改过就先核这一层，再改钉的 sha256）", file=sys.stderr)
    sys.exit(EXIT_WRONG_PINNED_FILE)


def broken(item: str) -> bool:
    return harness.broken(BREAK_VARIABLE, item)


def resolve(repo: str, path: str) -> str:
    return path if os.path.isabs(path) else os.path.join(repo, path)


# ── 修订 11 占位 ──
def load_reuse_config(path: str) -> dict:
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


@dataclasses.dataclass
class ReuseState:
    """这一层的复用状态：标记目录、状态目录的上一层、每个 (单元, 第几次) 那一份计划、算请求用的那棵树（第一次用到时现算）。"""
    repo: str
    markers: str
    state_parent: str
    runs: dict
    facts: reuse.TreeFacts | None = None


STATE: dict = {"reuse": None}
ORIGINAL = {"unit_table": units8.unit_table, "unit_environment": units8.unit_environment, "finish_unit": units8.finish_unit,  # 逐个点名留下原件
            "run": units8.run, "plan": units8.plan}


# ── 换进修订 8 单元编排的几处（configure 一次装好；那边的函数按模块全局名取，换了就用这里的）──
def unit_table_with_selfproof() -> dict:
    """修订 8 的单元表加 reuse-selfproof（RP2，依赖 step-0；只打 S1 常量的三趟 stream-constants）。"""
    units = ORIGINAL["unit_table"]()
    units[SELFPROOF_UNIT] = units8.Unit(SELFPROOF_UNIT, ("step-0",), units8.registered_threads(), 300)  # 同修订 8 每个单元：取整份额度（第 2 次重跑登记修订 11）
    return units


def tree_facts(settings) -> reuse.TreeFacts:
    """算请求用的那棵树（第一次用到时现算一次）。"""
    state = STATE["reuse"]
    if state.facts is None:
        state.facts = reuse.tree_facts(settings.repo, settings.tree, dict(os.environ))
    return state.facts


def marker_roles(unit_name: str) -> dict[str, str]:
    """一个读数单元结束时记进标记的角色路径（相对仓根）。"""
    marker = reuse.read_markers(STATE["reuse"].markers).get(unit_name, {})
    return {key[len("role_"):]: value for key, value in marker.items() if key.startswith("role_")}


def upstream_for(settings, reading) -> list[tuple[str, str]]:
    located = []
    for role, name in reading.upstream:
        roles = marker_roles(name)
        location = roles.get("directory") or roles.get("row")
        if not location:
            raise units8.RunCondition(f"{reading.name} 的上游 {name} 没有记下读数的位置（标记里没有角色路径）")
        located.append((role, resolve(settings.repo, location)))
    return located


def step_4_limit(settings) -> int:
    """第 4 步各行的 L：step-1 那份计时的 process_wall（驱动 --baseline-timing 给的就是它），max(1800, ⌈3T⌉)。"""
    timing = marker_roles("step-1").get("timing")
    if not timing:
        raise units8.RunCondition("step-1 的计时没有记下（标记里没有角色路径），第 4 步的 L 算不出")
    baseline = device_timing.read_baseline_timing(open(resolve(settings.repo, timing), encoding="utf-8").read().splitlines())
    return device_timing.row_time_limit_seconds(baseline.process_wall_seconds, False)


def request_for(settings, reading) -> reuse.UnitRequest:
    """一个读数单元的请求（reuse_library 文件头「按项判定」）：上游取上游那一单元记下的读数，第 4 步各行的 L 取 step-1 计时。"""
    limit = step_4_limit(settings) if isinstance(reading, reuse.RowUnit) else None
    return reuse.unit_request(tree_facts(settings), reading, reuse.selected_row_lines(settings.repo), limit, tuple(upstream_for(settings, reading)))


def environment_with_reuse(settings, unit, attempt: int, directory: str, names: dict[str, str], print_only: bool) -> dict[str, str]:
    """修订 8 的 unit_environment 加 REUSE_LIBRARY（这一份计划的状态目录）、REUSE_ENTRIES、REUSE_READINGS、REUSE_STEPS；真起之前（不是 PRINT_ONLY）
    这个单元装的读数单元一份计划、写 reuse-entries.tsv。没有读数单元的（reuse-selfproof）REUSE_LIBRARY 给单元自己的一个空目录（RP2 的状态目录放它底下）。"""
    environment = ORIGINAL["unit_environment"](settings, unit, attempt, directory, names, print_only)
    state = STATE["reuse"]
    readings = reuse.readings_of_owner(unit.name)
    own = os.path.join(state.state_parent, f"{unit.name}-{attempt}")
    environment.update(REUSE_LIBRARY=own, REUSE_ENTRIES=os.path.join(directory, "reuse-entries.tsv"),
                       REUSE_READINGS=os.path.join(directory, "reuse-readings.tsv"), REUSE_STEPS=",".join(reading.step_name for reading in readings))
    if print_only:
        return environment
    os.makedirs(own, exist_ok=True)
    if not readings:
        return environment
    run = reuse.ItemRun(settings.repo, SCRIPT_KEY, [request_for(settings, reading) for reading in readings], own, state.markers,
                        printer=lambda line: units8.say(settings, line))
    decisions = run.plan()
    state.runs[(unit.name, attempt)] = run
    environment["REUSE_LIBRARY"] = run.state
    with open(os.path.join(directory, "reuse-entries.tsv"), "x", encoding="utf-8") as handle:
        handle.writelines(f"{reading.step_name}\t{decisions[reading.name].item}\n" for reading in readings
                          if decisions[reading.name].state == reuse.STATE_REUSED)
    taken = sum(1 for decision in decisions.values() if decision.state == reuse.STATE_REUSED)
    units8.say(settings, f"  复用 {unit.name}：读数单元 {len(decisions)} 个，取来 {taken} 个" +
               "".join(f"；{decision.unit} {decision.state}（{decision.reason}）" for decision in decisions.values()))
    return environment


def finish_with_insertion(settings, item, intervals) -> bool:
    """修订 8 的 finish_unit（绿记绿行与清单、红记一行），再把单元脚本记下的每一步照跑出来的读数照入库条件判绿或判红、照 RC1、RC2 比，这一份计划收尾；
    S18 触发就记红。"""
    state = STATE["reuse"]
    lines = reuse.readings_file_lines(os.path.join(item.directory, "reuse-readings.tsv"))
    states = [reading_state for _, reading_state, _, _ in lines]
    green = ORIGINAL["finish_unit"](settings, item, intervals)
    run = state.runs.pop((item.unit.name, item.attempt), None)
    by_step = {reading.step_name: reading for reading in reuse.readings_of_owner(item.unit.name)}
    try:
        for step, reading_state, exit_text, roles in lines:
            unit = by_step.get(step)
            decision = run.decisions.get(unit.name) if run is not None and unit is not None else None
            if decision is None:
                continue
            reading = {role: resolve(settings.repo, roles[role]) for role in reuse.READING_ROLES if role in roles}
            reuse.write_marker(state.markers, decision, {f"role_{role}": os.path.relpath(path, settings.repo) for role, path in reading.items()})
            if reading_state != "fresh":
                continue
            if isinstance(unit, reuse.RowUnit):
                insertable = reuse.row_insertable(reading["row"])
                fields = reuse.row_result_fields(reading["row"])
                exit_code = int(fields["device_exit"]) if fields.get("device_exit", "-").lstrip("-").isdigit() else -1
            else:
                insertable = reuse.device_exit_insertable(int(exit_text), unit.expected_exits) if exit_text.lstrip("-").isdigit() else (False, "no-exit")
                exit_code = int(exit_text) if exit_text.lstrip("-").isdigit() else -1
            outcome = run.record(unit.name, reading, exit_code, insertable)
            units8.say(settings, f"  入库 {decision.unit}：{outcome}（{insertable[1] or '入库条件过'}）")
    finally:
        if run is not None:
            run.finish()
        shutil.rmtree(os.path.join(state.state_parent, f"{item.unit.name}-{item.attempt}"), ignore_errors=True)
    problems = reuse.s18_problems(state.markers)
    if problems and not broken("s18-ignored"):
        units8.say(settings, f"✗ S18 触发（同一把键读数不同或复用哨兵红，{len(problems)} 行事件，第一行：{problems[0][:200]}）：那一把键已撤成红，"
                             f"单元 {item.unit.name} 记红，交主 agent")
        return False
    if states:
        units8.say(settings, f"  单元 {item.unit.name} 的读数：取来 {states.count(reuse.STATE_REUSED)} 个、照跑 {states.count('fresh')} 个；判法照跑")
    return green


def registered_readings(settings) -> list[str]:
    """账本登记的读数单元：这一趟要做的单元装的（research-mutations 排了才算；reuse-selfproof 不算，它的产物是它自己的那一份）。"""
    units = unit_table_with_selfproof()
    done, _ = units8.resume_state(settings, units)
    owners = [name for name in units if name != SELFPROOF_UNIT and (name != "research-mutations" or units8.research_side_differs(settings, done))]
    return [reading.name for owner in owners for reading in reuse.readings_of_owner(owner)]


def run_with_ledger(settings) -> int:
    """修订 8 的 run，收尾现合复用账本：跑完（退 0）装进 research/results，没跑完只写日志目录；N + M 不等于登记数退 1。"""
    code = ORIGINAL["run"](settings)
    text, covers = reuse.ledger_text(STATE["reuse"].markers, registered_readings(settings))
    os.makedirs(settings.logs, exist_ok=True)
    with open(os.path.join(settings.logs, f"{LEDGER_STEM}.out"), "w", encoding="utf-8") as handle:
        handle.write(text)
    if code == 0:
        installed = reuse.install_ledger(os.path.join(settings.repo, settings.product(LEDGER_STEM)), text)
        units8.say(settings, f"  复用账本 {os.path.relpath(installed, settings.repo)}：{text.strip().splitlines()[-1]}")
        if not covers:
            units8.say(settings, "✗ 复用账本 N + M 不等于登记的读数单元数（第 11.4 节判红）")
            return 1
    return code


def plan_with_reuse(settings) -> int:
    """修订 8 的 plan 原样，再打每个读数单元的键与库里有没有判绿的（只读：不写标记、不写库）。"""
    code = ORIGINAL["plan"](settings)
    print("── 复用（修订 11）──")
    print(f"判定库 prune-server apparatus-items/{reuse.KIND}；标记 {STATE['reuse'].markers}")
    try:
        facts = tree_facts(settings)
    except OSError as error:   # 冻结树还没拍、读不到：同第二段编排层那一句，不打 traceback（第 2 次重跑登记修订 4）
        print(f"✗ 算请求用的树 {settings.tree} 读不到：{error}")
        print('     → 怎么办：拍好修订 8 那棵冻结树（H3）再起')
        return code
    for name in registered_readings(settings):
        reading = [unit for unit in list(reuse.SEGMENT_ONE_READING_UNITS) + reuse.step_4_row_units() if unit.name == name][0]
        waiting = [upstream for _, upstream in reading.upstream if not marker_roles(upstream)]
        if waiting or (isinstance(reading, reuse.RowUnit) and not marker_roles("step-1")):
            print(f"读数单元 {name}：键待上游（{','.join(waiting) or 'step-1 的计时'}）")
            continue
        request = reuse.unit_request(facts, reading, reuse.selected_row_lines(settings.repo),
                                     step_4_limit(settings) if isinstance(reading, reuse.RowUnit) else None, tuple(upstream_for(settings, reading)))
        key, why = reuse.keyed_items(settings.repo, [request])[name]
        if request.reason or key is None:
            print(f"读数单元 {name}：拒算（{request.reason or why}），照跑")
            continue
        print(f"读数单元 {name}：键 {key[:16]}；{'库里有判绿的' if reuse.stored_value(settings.repo, key) is not None else '库里没有判绿的'}")
    return code


def configure(repo: str, config: dict, plan: bool, dumps: str) -> None:
    """装好复用状态与换进修订 8 的几处。"""
    state_parent = os.path.join(repo, dumps, "reuse-items", "units")
    if not plan:
        shutil.rmtree(state_parent, ignore_errors=True)   # 状态目录只属于一趟：上一趟停在中途留下的不读、删掉
    STATE["reuse"] = ReuseState(repo, os.path.join(repo, dumps, "reuse-markers", "units"), state_parent, {})
    units8.UNIT_SCRIPT, units8.ROWS_ORCHESTRATOR = UNIT_SCRIPT, ROWS_ORCHESTRATOR
    units8.unit_table = unit_table_with_selfproof
    units8.unit_environment = environment_with_reuse
    units8.finish_unit = finish_with_insertion


def main(argument_list: list[str]) -> int:
    for variable, selftest_variable in ((BREAK_VARIABLE, SELFTEST_VARIABLE), (reuse.BREAK_VARIABLE, reuse.SELFTEST_VARIABLE),
                                        (units8.BREAK_VARIABLE, units8.SELFTEST_VARIABLE)):
        refusal = harness.refuse_break_outside_selftest(variable, selftest_variable)
        if refusal:
            print(refusal, file=sys.stderr)
            return 2
    if argument_list[:1] == ["--selftest"]:
        return selftest(argument_list[1:])
    parser = argparse.ArgumentParser(description="E171 修订 11：修订 8 那一趟第 0–10 步按单元编排，带复用")
    parser.add_argument("mode", choices=("run", "plan"))
    parser.add_argument("--config", required=True, help="修订 11 占位（revision_11_placeholders.json 那一形）")
    parser.add_argument("--repo", default=os.getcwd())
    arguments = parser.parse_args(argument_list)
    repo, config_path = os.path.abspath(arguments.repo), os.path.abspath(arguments.config)
    try:
        config = load_reuse_config(config_path)
        settings = units8.settings_from_config(repo, resolve(repo, config["revision_8_config"]))
        configure(repo, config, arguments.mode == "plan", settings.dumps)
        os.environ[CONFIG_VARIABLE] = config_path
        if arguments.mode == "plan":
            print(f"修订 11 占位 {config_path}（dry_run={str(bool(config.get('dry_run'))).lower()}；{config.get('dry_run_note') or '-'}）；"
                  f"H31 {'；'.join(reuse.h31_problems(repo, config)) or '与现在的文件逐个相同'}")
            return plan_with_reuse(settings)
        if config.get("dry_run"):
            raise units8.RunCondition(f"修订 11 占位是干跑配置（{config.get('dry_run_note') or '没写说明'}），只给 plan 用")
        problems = [] if broken("h31-unchecked") else reuse.h31_problems(repo, config)
        if problems:
            raise units8.RunCondition("H31 不过：" + "；".join(problems[:4]))
        reuse.check_service_reachable(repo)
        return run_with_ledger(settings)
    except (units8.RunCondition, judge.RunConditionError, reuse.RunCondition) as error:
        print(f"{units8.today()} ✗ 运行条件不满足（退 78）：{error}", flush=True)
        return units8.EXIT_RUN_CONDITION


# ── 自证：假仓与假树取 reuse_library.fake_world（装置入口经 LONG_RUN_ITEMS_SELFTEST_ENTRIES 换成假入口）；假单元脚本照真脚本的取用协议
#    （REUSE_ENTRIES 取用、REUSE_READINGS 记读数），每一步之后跑一个假判法（取来的也跑）；逐行编排换成修订 8 自证那个假的 ──
FAKE_REUSE_UNIT_SCRIPT = r'''#!/bin/bash
set -u
unit=$1
if [ "${PRINT_ONLY:-0}" = 1 ]; then echo "  \$ fake-reuse-unit $unit steps=${REUSE_STEPS:-}"; exit 0; fi
echo "start $unit ${TAG:--}" >> "$FAKE_LOG"
if [ "$unit" = step-0 ]; then
  frozen="$PREFIX-frozen-tree$TAG$SUFFIX.out"
  printf 'E7RESULT name=verdict research_side_differs=false triggered=none\n' > "$frozen"; printf '%s\t1\n' "$frozen" >> "$UNIT_PRODUCTS"
fi
IFS=',' read -r -a steps <<< "${REUSE_STEPS:-}"
for step in "${steps[@]}"; do
  identity=${step/baseline-run2/baseline-run1}; identity=${identity/-run-2/-run-1}   # 同一把键的两趟（Z1、Z7）读数相同
  entry=$(awk -F'\t' -v name="$step" '$1 == name {print $2}' "$REUSE_ENTRIES" 2>/dev/null)
  product="$PREFIX-$step$TAG$SUFFIX.out"; timing="$PREFIX-$step$TAG$SUFFIX-timing.out"; directory="$DUMPS/$step$TAG"
  case $step in *-driver) roles=("row=$REPO/$directory/row-0") ;; *) roles=("product=$product" "timing=$timing" "directory=$directory") ;; esac
  if [ -n "$entry" ]; then
    maps=(); for role in "${roles[@]}"; do maps+=(--map "$role"); done
    code=$(python3 -B "$REUSE_LIBRARY_SCRIPT" fetch --library "$REUSE_LIBRARY" --entry "$entry" "${maps[@]}") || { echo "fetch-failed $step" >> "$FAKE_LOG"; exit 1; }
    state=reused
  else
    code=0; state=fresh
    [ "$step" = positive-control-panic ] && code=3   # 同真单元：panic 对照的装置预期退 3
    salt=none; [ "${UNITS_R11_FAKE_SALT_STEP:-}" = "$step" ] && salt=1
    case $step in
      *-driver)
        mkdir -p "$directory/row-0/dump/s"
        echo "E7RESULT name=row step=$identity salt=$salt" > "$directory/row-0/device.out"; echo "$identity" > "$directory/row-0/dump/s/states.tsv"
        printf 'row_number\t0\nlane\t0\npackages_checked_rebuilt\tcrates/core\nsubstituted_from\t-\nfailed_attempts\t0\ncopy_ns\t%s\nstatus\tok\ndevice_exit\t0\n' "$RANDOM" > "$directory/row-0/row-result.tsv" ;;
      *)
        echo "E7RESULT name=fake step=$identity" > "$product"
        printf 'E7TIMING name=stream_time stream=a elapsed_ns=1000000000\nE7TIMING name=process_wall seconds=1%s exit=0\n' "$RANDOM" > "$timing"
        mkdir -p "$directory/s"; echo "$identity salt=$salt" > "$directory/s/states.tsv" ;;
    esac
  fi
  ( IFS=$'\t'; printf '%s\n' "$step"$'\t'"$state"$'\t'"$code"$'\t'"${roles[*]}" ) >> "$REUSE_READINGS"
  case $step in
    *-driver) judged=$(sha256sum < "$directory/row-0/row-result.tsv" | cut -c1-12) ;;
    *) judged=$(sha256sum < "$product" | cut -c1-12); printf '%s\t1\n%s\t%s\n' "$product" "$timing" "$(grep -c '' "$timing")" >> "$UNIT_PRODUCTS" ;;
  esac
  echo "$directory" >> "$UNIT_DIRECTORIES"
  echo "judge $unit $step $state $judged" >> "$FAKE_LOG"
  if [ "${UNITS_R11_FAKE_JUDGE_RED:-}" = "$step" ]; then echo "judge-red $step" >> "$FAKE_LOG"; exit 1; fi
done
echo "end $unit" >> "$FAKE_LOG"
'''


@dataclasses.dataclass
class WorldEleven:
    root: str
    repo: str
    environment: dict


def fake_world_eleven(root: str) -> WorldEleven:
    base = reuse.fake_world(os.path.join(root, "world"))
    reuse.selftest_entries(base.entries, reuse.segment_one_reading_names())
    script = judge.write_text(os.path.join(root, "fake-reuse-unit.sh"), FAKE_REUSE_UNIT_SCRIPT)
    rows = judge.write_text(os.path.join(root, "fake-rows.sh"), units8.FAKE_ROWS)
    checks = reuse.items_library.lib_reuse_checks
    environment = dict(os.environ, UNITS_R8_SELFTEST="1", UNITS_R11_SELFTEST="1", REUSE_SELFTEST="1", UNITS_R8_UNIT_SCRIPT=script, UNITS_R8_ROWS_ORCHESTRATOR=rows,
                       UNITS_R8_PREREQUISITES_COMMAND="true", UNITS_R8_POLL_SECONDS="0.02", FAKE_LOG=os.path.join(root, "fake.log"),
                       REUSE_LIBRARY_SCRIPT=os.path.join(HERE, "reuse_library.py"), PYTHONDONTWRITEBYTECODE="1",
                       **{reuse.items_library.LIBRARY_DIRECTORY_VARIABLE: os.path.join(root, "library"), reuse.items_library.SUBSTITUTION_VARIABLE: base.entries,
                          checks.SENTINEL_COUNT_VARIABLE: reuse.ALL_SENTINELS})
    return WorldEleven(root, base.repo, environment)


def h31_of_this_layer() -> dict[str, str]:
    return {path: reuse.sha256_file(path) for path in (os.path.abspath(__file__), UNIT_SCRIPT, os.path.join(HERE, "reuse_library.py"))}


def pass_configs(world: WorldEleven, name: str, h31: bool = True) -> str:
    """一遍的两份配置：修订 8 那一份（H16 各遍不同：日期、导出根、日志目录）与修订 11 那一份。交回修订 11 那一份的路径。"""
    revision_8 = {"dry_run": False, "tree": reuse.FAKE_TREE_RELATIVE, "H3": {"snapshot_digest": "f" * 64}, "H8": {"mutations_table_sha256": "e" * 64},
                  "H14": {"reverse_states": 72, "main_states": 1207959552}, "H7": {},
                  "H16": {"run_date": f"2026-10-{name}", "run_suffix": "-r8", "dumps_root": f"scripts/experiment-apparatus/target/e171-r1-dumps/fake-{name}",
                          "logs_directory": os.path.join(world.root, f"logs-{name}")}}
    revision_8_path = judge.write_text(os.path.join(world.root, f"revision-8-{name}.json"), json.dumps(revision_8))
    revision_11 = {"dry_run": False, "revision_8_config": revision_8_path, "H31": {"files": h31_of_this_layer() if h31 else None}}
    return judge.write_text(os.path.join(world.root, f"revision-11-{name}.json"), json.dumps(revision_11))


def run_pass(world: WorldEleven, config: str, extra: dict[str, str] | None = None) -> tuple[int, str]:
    completed = subprocess.run([sys.executable, "-B", os.path.abspath(__file__), "run", "--config", config, "--repo", world.repo], cwd=world.repo,
                               env=dict(world.environment, **(extra or {})), capture_output=True, text=True, timeout=600)
    return completed.returncode, completed.stdout + completed.stderr


def ledger_verdict(world: WorldEleven, name: str) -> str:
    path = os.path.join(world.repo, f"research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-{name}-s1-{LEDGER_STEM}-r8.out")
    return open(path, encoding="utf-8").read().strip().splitlines()[-1] if os.path.exists(path) else "账本不在"


def fake_log_lines(world: WorldEleven, start: int = 0) -> list[str]:
    path = world.environment["FAKE_LOG"]
    return open(path, encoding="utf-8").read().splitlines()[start:] if os.path.exists(path) else []


def cell_reuse_end_to_end(scratch: str) -> tuple[bool, str]:
    """第一遍库空（全部现判、入库）；第二遍：除 step-9、step-10-run-2（second-run，拒算）外全部取来，每一步的假判法照跑，账本 reused=30、
    没有 S18；第三遍、假判法在一个取来的单元（step-1）上判红：那个单元红、整趟退 1（取来的单元判法照跑，判红照停）。"""
    world = fake_world_eleven(scratch)
    first_code, first_output = run_pass(world, pass_configs(world, "11"))
    first_lines = len(fake_log_lines(world))
    second_code, second_output = run_pass(world, pass_configs(world, "12"))
    second = fake_log_lines(world, first_lines)
    judged = [line.split() for line in second if line.startswith("judge ")]
    fresh = sorted(words[2] for words in judged if words[3] != "reused")
    third_start = len(fake_log_lines(world))
    third_code, third_output = run_pass(world, pass_configs(world, "13"), {"UNITS_R11_FAKE_JUDGE_RED": "s2-device-baseline-streams-1-2"})
    third = fake_log_lines(world, third_start)
    first_verdict, second_verdict = ledger_verdict(world, "11"), ledger_verdict(world, "12")
    # 账本末行的字段按「 名=值 」整段认：末行最后一个字段（sentinel_problems）后面没有空格，两头各补一个再找
    first_fields, second_fields = f" {first_verdict} ", f" {second_verdict} "
    checks = {
        "第一遍退 0": first_code == 0,
        "第一遍账本 reused=0、units_cover_registered=true、check_mismatches=0、sentinel_problems=0":
            all(f" {field} " in first_fields for field in ("reused=0", "units_cover_registered=true", "check_mismatches=0", "sentinel_problems=0")),
        "第二遍退 0": second_code == 0,
        "第二遍判法跑了 32 步": len(judged) == 32,
        "第二遍照跑的只有 baseline-run2 与 Q5 主取样点第 2 次": fresh == ["baseline-run2", "q5-main-before-the-first-barrier-of-call-3-run-2"],
        "第二遍账本 reused=30、check_mismatches=0、sentinel_problems=0":
            all(f" {field} " in second_fields for field in ("reused=30", "check_mismatches=0", "sentinel_problems=0")),
        "第三遍退 1": third_code == 1,
        "第三遍 step-1 取来、判法照跑且判红": "judge-red s2-device-baseline-streams-1-2" in third
                                             and any(line.startswith("judge step-1 ") and " reused " in line for line in third),
    }
    failed = [label for label, held in checks.items() if not held]
    return not failed, (f"不过的 {failed or '无'}；第一遍退 {first_code}（{first_verdict[22:]}）；第二遍退 {second_code}，判法跑了 {len(judged)} 步（要 32），"
                        f"照跑的 {fresh}，账本 {second_verdict[22:]}；第三遍（取来的 step-1 判红）退 {third_code}；第三遍末尾 {third_output.strip()[-200:]}")


def cell_h31_refused(scratch: str) -> tuple[bool, str]:
    world = fake_world_eleven(scratch)
    code, output = run_pass(world, pass_configs(world, "21", h31=False))
    return code == 78 and not fake_log_lines(world) and "H31" in output, f"H31 没填：退 {code}，假单元起过 {bool(fake_log_lines(world))}"


def cell_s18_stops(scratch: str) -> tuple[bool, str]:
    """step-9 与 step-7 同一把键（step-9 是 second-run）；step-9 的导出掺一个字：后入库的那一个与库里先入的那一份读数不同（RC2，或单元并发时 RC1 insert）
    → S18：那一把键撤成红、后跑完的那个单元红、整趟退 1。"""
    world = fake_world_eleven(scratch)
    code, output = run_pass(world, pass_configs(world, "31"), {"UNITS_R11_FAKE_SALT_STEP": "baseline-run2"})
    events = reuse.event_lines(os.path.join(world.repo, "scripts/experiment-apparatus/target/e171-r1-dumps/fake-31/reuse-markers/units"))
    mismatched = [line for line in events if " name=reuse_check " in f" {line} " and " identical=false " in f" {line} "]
    return code == 1 and "S18 触发" in output and bool(mismatched), f"step-9 掺一个字：退 {code}；读数不同 {len(mismatched)} 行；输出末尾 {output.strip()[-200:]}"


STEP_1_VERDICT = ("E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 s3_o2_judgement_mismatches=0 "
                  "s9_failed_checks=0 streams_not_exported=0 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o1_reference_filesystem_red_states=0 "
                  "o2_red_states={o2} recovery_panic_states=0\n")


def real_script_step_1(scratch: str, label: str, world, o2_red_states: int) -> tuple[int, list, str, str]:
    """真的 segment_one_unit_revision_11.sh 跑 step-1：编排给的状态目录里那一项取来的产物 o2_red_states 取给定值（状态目录照 long_run_item_verdicts 的 plan
    放取来的那一份的样子摆：outputs/<项>.out 与 .exit）。交回 (退出码, 记下的读数, 产物路径, 那一份产物)。"""
    source = os.path.join(scratch, f"reading-{label}")
    reading = {"product": judge.write_text(os.path.join(source, "product"), STEP_1_VERDICT.format(o2=o2_red_states)),
               "timing": judge.write_text(os.path.join(source, "timing"), "E7TIMING name=stream_time stream=a elapsed_ns=1000000000\n"
                                                                          "E7TIMING name=process_wall seconds=2.000 exit=0\n"),
               "directory": os.path.dirname(judge.write_text(os.path.join(source, "export", "s", "states.tsv"), "states\n"))}
    item = "step-1"
    state = os.path.join(scratch, f"state-{label}")
    os.makedirs(os.path.join(state, reuse.items_library.OUTPUTS_DIRECTORY))
    with open(os.path.join(state, reuse.items_library.OUTPUTS_DIRECTORY, item + ".out"), "wb") as handle:
        handle.write(reuse.packed_reading(reading))
    judge.write_text(os.path.join(state, reuse.items_library.OUTPUTS_DIRECTORY, item + ".exit"), "0\n")
    unit_directory = os.path.join(scratch, f"unit-{label}")
    os.makedirs(unit_directory)
    entries = judge.write_text(os.path.join(unit_directory, "reuse-entries.tsv"), f"s2-device-baseline-streams-1-2\t{item}\n")
    tree = os.path.relpath(world.tree, world.repo)
    environment = dict(os.environ, REPO=world.repo, RESEARCH=HERE, PREFIX=f"research/results/fake-{label}", SUFFIX="-r8", TAG="", DUMPS=f"scripts/experiment-apparatus/target/fake-dumps-{label}",
                       TREE=tree, BIN="/bin/false", E170_BIN="/bin/false", SNAPSHOT_DIGEST=judge.row_driver.snapshot_digest(world.tree)[0],
                       MUTATIONS_SHA256=reuse.sha256_file(os.path.join(world.tree, "crates/mutations.tsv")), CONFIG="-", THREADS="1",
                       WORK_ROOT=os.path.join(unit_directory, "work"), UNIT_LOG=os.path.join(unit_directory, "unit.log"),
                       UNIT_PRODUCTS=os.path.join(unit_directory, "products.tsv"), UNIT_DIRECTORIES=os.path.join(unit_directory, "directories.tsv"),
                       REUSE_LIBRARY=state, REUSE_ENTRIES=entries, REUSE_READINGS=os.path.join(unit_directory, "reuse-readings.tsv"), PRINT_ONLY="0")
    code = subprocess.run(["bash", UNIT_SCRIPT, "step-1"], cwd=world.repo, env=environment, capture_output=True, text=True, timeout=120).returncode
    product = os.path.join(world.repo, f"research/results/fake-{label}-s2-device-baseline-streams-1-2-r8.out")
    return code, reuse.readings_file_lines(environment["REUSE_READINGS"]), product, reading["product"]


def cell_unit_script_fetch(scratch: str) -> tuple[bool, str]:
    """真的单元脚本：编排给了状态目录与项名就取用（照角色解到这一趟的名字、不起装置），记一行 reused，判法（check_product.py verdict、
    device_timing.py check）照跑；取来的产物不合判法（o2_red_states=1）时单元红。"""
    world = reuse.fake_world(os.path.join(scratch, "world"))
    good_code, good_lines, good_product, good_source = real_script_step_1(scratch, "good", world, 0)
    red_code, red_lines, _, _ = real_script_step_1(scratch, "red", world, 1)
    same = os.path.exists(good_product) and open(good_product, "rb").read() == open(good_source, "rb").read()
    states = [(step, state, roles.keys() and sorted(roles)) for step, state, _, roles in good_lines]
    good = good_code == 0 and same and states == [("s2-device-baseline-streams-1-2", "reused", ["directory", "product", "timing"])] \
        and red_code == 1 and [state for _, state, _, _ in red_lines] == ["reused"]
    return good, f"合判法的那一份：退 {good_code}，产物与那一份逐字节相同 {same}，记下 {states}；不合判法的：退 {red_code}"


def cell_unit_script_driver_report_only(scratch: str) -> tuple[bool, str]:
    """真的单元脚本 PRINT_ONLY=1 打第 4 步一项（step-4-o2）会跑的命令：编排给了那一步驱动行的项名的，先打取用那一条、起驱动那一条带 --report-only；
    没给的照现跑起驱动、不带。"""
    world = reuse.fake_world(os.path.join(scratch, "world"))
    reading = [unit for unit in reuse.readings_of_owner("step-4-o2") if unit.step_name == "empty-control-o2-driver"][0]
    printed = {}
    for label, entries in (("fetched", f"{reading.step_name}\t{reuse.item_name(reading.name)}\n"), ("fresh", "")):
        unit_directory = os.path.join(scratch, f"unit-{label}")
        environment = dict(os.environ, REPO=world.repo, RESEARCH=HERE, PREFIX=f"research/results/fake-{label}", SUFFIX="-r8", TAG="",
                           DUMPS=f"scripts/experiment-apparatus/target/fake-dumps-{label}", TREE=os.path.relpath(world.tree, world.repo),
                           S2_BASELINE="s2-device-baseline-streams-1-2", WORK_ROOT=os.path.join(unit_directory, "work"),
                           UNIT_LOG=os.path.join(unit_directory, "unit.log"), UNIT_PRODUCTS=os.path.join(unit_directory, "products.tsv"),
                           UNIT_DIRECTORIES=os.path.join(unit_directory, "directories.tsv"), REUSE_LIBRARY=os.path.join(unit_directory, "state"),
                           REUSE_ENTRIES=judge.write_text(os.path.join(unit_directory, "reuse-entries.tsv"), entries),
                           REUSE_READINGS=os.path.join(unit_directory, "reuse-readings.tsv"), PRINT_ONLY="1")
        completed = subprocess.run(["bash", UNIT_SCRIPT, "step-4-o2"], cwd=world.repo, env=environment, capture_output=True, text=True, timeout=120)
        printed[label] = (completed.returncode, [line for line in completed.stdout.splitlines() if "row_driver.py" in line],
                          [line for line in completed.stdout.splitlines() if "reuse_library.py fetch" in line])
    fetched_code, fetched_drivers, fetched_fetches = printed["fetched"]
    fresh_code, fresh_drivers, fresh_fetches = printed["fresh"]
    good = (fetched_code == 0 and len(fetched_fetches) == 1 and len(fetched_drivers) == 1 and "--report-only" in fetched_drivers[0].split()
            and fresh_code == 0 and not fresh_fetches and len(fresh_drivers) == 1 and "--report-only" not in fresh_drivers[0].split())
    return good, (f"取来的：退 {fetched_code}，取用 {len(fetched_fetches)} 条，驱动带 --report-only "
                  f"{[('--report-only' in line.split()) for line in fetched_drivers]}；现跑的：退 {fresh_code}，取用 {len(fresh_fetches)} 条，"
                  f"驱动带 --report-only {[('--report-only' in line.split()) for line in fresh_drivers]}")


def cell_reuse_three_questions(scratch: str) -> tuple[bool, str]:
    """复用三问与复用哨兵的六格（reuse_library.run_reuse_cells）：这一层的读数单元 step-1、step-7，规范输出 REUSE_SELFTEST script=<本文件>。"""
    return reuse.run_reuse_cells(scratch, SCRIPT_KEY, ("step-1", "step-7"))


def selftest_cell_functions():
    """自证的各格，格名到函数。表写在函数体里：写成模块级赋值，会在 import 这个模块时把全部自证格连同它们起的子进程拖进 import 一方的判法单元闭包。"""
    return {"reuse-e2e": cell_reuse_end_to_end, "unit-script-fetch": cell_unit_script_fetch,
            "unit-script-driver-report-only": cell_unit_script_driver_report_only, "h31-refused": cell_h31_refused,
            "s18-stops": cell_s18_stops, "reuse-three-questions": cell_reuse_three_questions}


SELFTEST_SWITCHES = {"h31-unchecked": "h31-refused", "s18-ignored": "s18-stops", "report-only-dropped": "unit-script-driver-report-only"}


def selftest(argument_list: list[str]) -> int:
    problem = reuse.prune_server_client.selftest_service_problem()
    if problem:   # 判定库只在 prune-server 里：自证只在临时服务上跑，不写真服务（文件头「修订 11 占位」那一段末句）
        print(f"✗ segment_one_units_revision_11.py 自证没跑：{problem}", file=sys.stderr)
        print(f"     → 怎么办：{reuse.prune_server_client.SELFTEST_SERVICE_REMEDY}", file=sys.stderr)
        return 2
    os.environ[SELFTEST_VARIABLE] = "1"
    return harness.run_selftest(os.path.abspath(__file__), argument_list, harness.cells_reporting_progress(selftest_cell_functions()), SELFTEST_SWITCHES,
                                BREAK_VARIABLE, SELFTEST_VARIABLE)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
