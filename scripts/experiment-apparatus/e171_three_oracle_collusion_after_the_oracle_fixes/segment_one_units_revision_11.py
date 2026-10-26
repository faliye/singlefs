#!/usr/bin/env python3
"""E171 第 1 次重跑·修订 11：修订 8 那一趟第 0–10 步按单元编排，带复用（登记 research/prompts/e171-r1-prereg.md 修订 11 第 11.4、11.5、11.9、11.10 节
与其后「主 agent 认定：修订 11 交回」）。由 run_segment_one_revision_11_multi_host.sh 起，在仓根跑。

为什么另写一层：segment_one_units_revision_8.py 与单元脚本 segment_one_unit_revision_8.sh 一个字节不改（前者改成线程额度取登记值之后按现内容重钉了
UNITS_R8_SHA256，原钉 cc52fcfc…）。这一层按路径载入前者（开跑先核 sha256，
不是就退 3），单元表、依赖、次序与资源、续跑、一步停、逐行编排的起法都调它原来的函数；改的只有第 11.10 节表那几处：
  ① 单元脚本换成 segment_one_unit_revision_11.sh（修订 8 那份照抄，只在读数进程那几处加取用），逐行编排换成 rows_orchestrator_revision_11.py
    （修订 11 占位的路径经环境变量 E171_REVISION_11_CONFIG 交给它）；
  ② 加单元 reuse-selfproof（RP2，依赖 step-0）；
  ③ 起一个单元之前（unit_environment）：这个单元装的每个读数单元（reuse_library.readings_of_owner）照第 11.2 节算键、查库，写标记
    （<H16 导出根>/reuse-markers/units/），命中的写进单元目录的 reuse-entries.tsv 交单元脚本取用；永远现跑的（step-9、step-10-run-2）不查库；
    上游读数的位置取上游那一单元的标记（它结束时记下的角色路径），第 4 步各行的 L 取 step-1 计时的 process_wall（max(1800, ⌈3T⌉)）；
  ④ 单元结束时（finish_unit）：单元脚本记下的每一步读数照入库条件入库（判法红不红不影响入库）；库里已有同键条目的照 RC1（第二趟 RC2）比；
    现跑的第 4 步驱动行先照 RC4 核那一次编译的 dep-info（不符就不入库）；S18 触发（账本事件里有 identical=false 或 matches=false）就冻住库、
    这个单元记红（修订 8 的一步停：不再起新单元、写停派文件、逐行编排不再派新行），交主 agent；
  ⑤ 收尾：由标记现合复用账本 <H16 前缀>-reuse-ledger-units<H16 后缀>.out（登记的读数单元：第 0–10 步各单元装的，research-mutations 排了才算），
    N + M 不等于登记数退 1；没跑完时账本只写进日志目录 reuse-ledger-units.out。
修订 11 占位（revision_11_placeholders.json，路径不是绝对的相对仓根）：revision_8_config、H30（library_root、lookup）、H31（新文件 → sha256：
  run 之前核，没填或对不上退 78；主 agent 认过才起跑）；dry_run=true 的只给 plan。库目录写不了、与 scripts/experiment-apparatus/target 不在同一个文件系统退 78。
用法：
  segment_one_units_revision_11.py run  --config <修订 11 占位> [--repo R]
  segment_one_units_revision_11.py plan --config <修订 11 占位> [--repo R]   修订 8 的 plan 全部原样，另打每个读数单元的键与库里有没有（只读，不建库、不写标记）
  segment_one_units_revision_11.py --selftest [--cells 格,…] [--rounds N]
退出码：同修订 8 单元编排（0、1、2、78、143）；另 3：segment_one_units_revision_8.py 不是钉的那一份；账本 N + M 不等于登记数退 1。

自证只换假件的变量（只在 UNITS_R11_SELFTEST=1 时认，另带修订 8 的 UNITS_R8_* 与 reuse_library 的 REUSE_SELFTEST_*）：UNITS_R11_FAKE_* 交给假单元脚本。
弄坏开关 UNITS_R11_BREAK=<项>（逗号连几个；只在 --selftest 起的子进程里认）。项名、弄坏哪一处、打开后红在哪一格：
  h31-unchecked          不核 H31（没填也起跑）                         h31-refused
  s18-ignored            S18 触发了照样记绿、库不冻                     s18-stops
  sweep-skipped、temporary-kept（自证外壳 revision_8_selftest_harness.py 的两个）     no-leftover-processes、temporary-directory-removed
"""
from __future__ import annotations

import argparse
import dataclasses
import importlib.util
import json
import os
import shlex
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
UNITS_R8_SHA256 = "be014e3334d494e4f947ef3de367461a5d2c48fe5fdb01a4be0facefe3e74537"
UNIT_SCRIPT = os.path.join(HERE, "segment_one_unit_revision_11.sh")
ROWS_ORCHESTRATOR = os.path.join(HERE, "rows_orchestrator_revision_11.py")
EXIT_WRONG_PINNED_FILE = 3
SELFPROOF_UNIT = "reuse-selfproof"
LEDGER_STEM = "reuse-ledger-units"


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
    """这一层的复用状态：配置、库、标记目录、读数单元的决定（(单元, 第几次) → 步名 → 决定）、算键用的那棵树（第一次用到时现算）。"""
    repo: str
    library: reuse.Library
    lookup: bool
    markers: str
    decisions: dict
    inputs: reuse.TreeInputs | None = None
    inputs_error: str = ""


STATE: dict = {"reuse": None}
ORIGINAL = {"unit_table": units8.unit_table, "unit_environment": units8.unit_environment, "finish_unit": units8.finish_unit,  # 逐个点名留下原件
            "run": units8.run, "plan": units8.plan}


# ── 换进修订 8 单元编排的几处（configure 一次装好；那边的函数按模块全局名取，换了就用这里的）──
def unit_table_with_selfproof() -> dict:
    """修订 8 的单元表加 reuse-selfproof（RP2，依赖 step-0；只打 S1 常量的三趟 stream-constants，内存照 CAP_SMALL）。"""
    units = ORIGINAL["unit_table"]()
    units[SELFPROOF_UNIT] = units8.Unit(SELFPROOF_UNIT, ("step-0",), units8.THREADS, units8.bytes_of(units8.CAP_SMALL), 300)
    return units


def tree_inputs(settings) -> reuse.TreeInputs | None:
    """算键用的那棵树（第一次用到时现算一次）；dep-info 里有树外路径（S18 ②）时交回 None，原因记进状态。"""
    state = STATE["reuse"]
    if state.inputs is None and not state.inputs_error:
        try:
            state.inputs = reuse.tree_inputs_for_run(settings.repo, settings.tree, dict(os.environ))
        except reuse.ClosureOutsideTheTree as error:
            state.inputs_error = str(error)
    return state.inputs


def marker_roles(unit_name: str) -> dict[str, str]:
    """一个读数单元结束时记进标记的角色路径（相对仓根）。"""
    marker = reuse.read_markers(STATE["reuse"].markers).get(unit_name, {})
    return {key[len("role_"):]: value for key, value in marker.items() if key.startswith("role_")}


def upstream_for(settings, reading) -> list[str]:
    located = []
    for role, name in reading.upstream:
        roles = marker_roles(name)
        location = roles.get("directory") or roles.get("row")
        if not location:
            raise units8.RunCondition(f"{reading.name} 的上游 {name} 没有记下读数的位置（标记里没有角色路径）")
        located.append((role, name, resolve(settings.repo, location)))
    return reuse.upstream_lines(located)


def step_4_limit(settings) -> int:
    """第 4 步各行的 L：step-1 那份计时的 process_wall（驱动 --baseline-timing 给的就是它），max(1800, ⌈3T⌉)。"""
    timing = marker_roles("step-1").get("timing")
    if not timing:
        raise units8.RunCondition("step-1 的计时没有记下（标记里没有角色路径），第 4 步的 L 算不出")
    baseline = device_timing.read_baseline_timing(open(resolve(settings.repo, timing), encoding="utf-8").read().splitlines())
    return device_timing.row_time_limit_seconds(baseline.process_wall_seconds, False)


def decide_reading(settings, reading) -> reuse.Decision:
    """照第 11.2 节算键、查库（第 11.4 节：永远现跑的不查），写标记。闭包认不全（S18 ②）时现跑、记一行事件，单元结束时停。"""
    state = STATE["reuse"]
    inputs = tree_inputs(settings)
    kind = "driver-row" if isinstance(reading, reuse.RowUnit) else reading.kind
    if inputs is None:
        decision = reuse.Decision(reading.name, kind, "-", "", reuse.STATE_FRESH, "closure-outside-the-tree")
        reuse.append_event(state.markers, f"E7RESULT name=reuse_closure_check unit={reuse.value(reading.name)} key=- matches=false "
                                          f"outside-the-tree={reuse.value(state.inputs_error[-120:])}")
        reuse.write_marker(state.markers, decision)
        return decision
    upstream = upstream_for(settings, reading)
    if isinstance(reading, reuse.RowUnit):
        sections, reason = reuse.row_sections(inputs, reading, reuse.selected_row_lines(settings.repo), reuse.limit_lines(step_4_limit(settings)), upstream)
    else:
        sections, reason = reuse.reading_unit_sections(inputs, reading, upstream), "second-run" if reading.fresh_always else ""
    decision, problems = reuse.decide(state.library, reading.name, kind, sections, reason, state.lookup)
    reuse.note_decision(state.markers, decision, problems)
    return decision


def environment_with_reuse(settings, unit, attempt: int, directory: str, names: dict[str, str], print_only: bool) -> dict[str, str]:
    """修订 8 的 unit_environment 加 REUSE_LIBRARY、REUSE_ENTRIES、REUSE_READINGS、REUSE_STEPS；真起之前（不是 PRINT_ONLY）逐个读数单元查库、写 reuse-entries.tsv。"""
    environment = ORIGINAL["unit_environment"](settings, unit, attempt, directory, names, print_only)
    state = STATE["reuse"]
    readings = reuse.readings_of_owner(unit.name)
    environment.update(REUSE_LIBRARY=state.library.root, REUSE_ENTRIES=os.path.join(directory, "reuse-entries.tsv"),
                       REUSE_READINGS=os.path.join(directory, "reuse-readings.tsv"), REUSE_STEPS=",".join(reading.step_name for reading in readings))
    if print_only or not readings:
        return environment
    decisions = {reading.step_name: decide_reading(settings, reading) for reading in readings}
    state.decisions[(unit.name, attempt)] = decisions
    with open(os.path.join(directory, "reuse-entries.tsv"), "x", encoding="utf-8") as handle:
        handle.writelines(f"{step}\t{decision.entry}\n" for step, decision in decisions.items() if decision.state == reuse.STATE_REUSED)
    taken = sum(1 for decision in decisions.values() if decision.state == reuse.STATE_REUSED)
    units8.say(settings, f"  复用 {unit.name}：读数单元 {len(decisions)} 个，取来 {taken} 个" +
               "".join(f"；{decision.unit} {decision.state}（{decision.reason}）" for decision in decisions.values()))
    return environment


def reading_of(unit_name: str, step: str):
    return [reading for reading in reuse.readings_of_owner(unit_name) if reading.step_name == step][0]


def finish_with_insertion(settings, item, intervals) -> bool:
    """修订 8 的 finish_unit（绿记绿行与清单、红记一行），再把单元脚本记下的每一步读数照入库条件入库；S18 触发就冻库、记红。"""
    state = STATE["reuse"]
    lines = reuse.readings_file_lines(os.path.join(item.directory, "reuse-readings.tsv"))
    states = [reading_state for _, reading_state, _, _ in lines]
    green = ORIGINAL["finish_unit"](settings, item, intervals)
    decisions = state.decisions.get((item.unit.name, item.attempt), {})
    for step, reading_state, exit_text, roles in lines:
        decision = decisions.get(step)
        if decision is None:
            continue
        reading = {role: resolve(settings.repo, path) for role, path in roles.items() if role != "compiled"}
        reuse.write_marker(state.markers, decision, {f"role_{role}": os.path.relpath(path, settings.repo) for role, path in reading.items()})
        if reading_state != "fresh" or decision.key == "-":
            continue
        unit = reading_of(item.unit.name, step)
        if isinstance(unit, reuse.RowUnit):
            compiled = resolve(settings.repo, roles["compiled"]) if "compiled" in roles else ""
            if compiled and not reuse.closure_check(state.markers, decision, os.path.join(compiled, "tree"),
                                                    os.path.join(compiled, "target/release", reuse.DEVICE_BINARY + ".d"),
                                                    reuse.mutated_closure_paths(tree_inputs(settings))):
                continue
            insertable = reuse.row_insertable(reading["row"])
        else:
            insertable = reuse.device_exit_insertable(int(exit_text), unit.expected_exits) if exit_text.lstrip("-").isdigit() else (False, "no-exit")
        meta = {"exit": exit_text, "origin_run": f"{settings.prefix}{settings.suffix}", "origin_product": f"{step}{units8.attempt_tag(item.attempt)}",
                "origin_host": "1", "threads": str(units8.THREADS)}
        outcome = reuse.record_finished(state.library, state.markers, decision, reading, insertable, meta)
        units8.say(settings, f"  入库 {decision.unit}：{outcome}（{insertable[1] or '入库条件过'}）")
    problems = reuse.s18_problems(state.markers)
    if problems and not broken("s18-ignored"):
        state.library.freeze(f"S18（修订 11 第 11.9 节）：{problems[0][:300]}")
        units8.say(settings, f"✗ S18 触发（同键读数不同或闭包认不全，{len(problems)} 行事件，第一行：{problems[0][:200]}）：库冻住、不再取不再入，"
                             f"单元 {item.unit.name} 记红，交主 agent")
        return False
    if states:
        units8.say(settings, f"  单元 {item.unit.name} 的读数：取来 {states.count(reuse.STATE_REUSED)} 个、现跑 {states.count('fresh')} 个；判法照跑")
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
    """修订 8 的 plan 原样，再打每个读数单元的键与库里有没有（只读：不建库、不写标记、不挪坏条目）。"""
    code = ORIGINAL["plan"](settings)
    state = STATE["reuse"]
    print("── 复用（修订 11）──")
    print(f"库 {state.library.root}（{'在' if os.path.isdir(state.library.root) else '还没建'}；冻住 {state.library.frozen_reason() or '否'}）；"
          f"查库 {'开' if state.lookup else '关'}；标记 {state.markers}")
    inputs = tree_inputs(settings)
    if inputs is None:
        print(f"✗ 算键用的树认不全（S18 ②）：{state.inputs_error}")
        print('     → 怎么办：把闭包里认不出的那几份（上面那句点名的）放进树里，或在登记里补上它们再起')
        return code
    for name in registered_readings(settings):
        reading = [unit for unit in list(reuse.SEGMENT_ONE_READING_UNITS) + reuse.step_4_row_units() if unit.name == name][0]
        waiting = [upstream for _, upstream in reading.upstream if not marker_roles(upstream)]
        if waiting or (isinstance(reading, reuse.RowUnit) and not marker_roles("step-1")):
            print(f"读数单元 {name}：键待上游（{','.join(waiting) or 'step-1 的计时'}）")
            continue
        if isinstance(reading, reuse.RowUnit):
            sections = reuse.row_sections(inputs, reading, reuse.selected_row_lines(settings.repo), reuse.limit_lines(step_4_limit(settings)),
                                          upstream_for(settings, reading))[0]
        else:
            sections = reuse.reading_unit_sections(inputs, reading, upstream_for(settings, reading))
        key = reuse.key_of(sections)
        present = os.path.isdir(state.library.entry_path(key))
        print(f"读数单元 {name}：键 {key[:16]}；{'永远现跑（RC2）' if reading.fresh_always else ('库里有同键条目' if present else '库里没有')}")
    return code


def configure(repo: str, config: dict) -> None:
    h30 = config.get("H30") or {}
    library = reuse.Library(h30.get("library_root") or reuse.DEFAULT_LIBRARY_ROOT)
    STATE["reuse"] = ReuseState(repo, library, bool(h30.get("lookup", True)), "", {})
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
        configure(repo, config)
        STATE["reuse"].markers = os.path.join(repo, settings.dumps, "reuse-markers", "units")
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
        STATE["reuse"].library.check_run_condition(repo)
        removed = STATE["reuse"].library.clear_stale_staging()
        if removed:
            units8.say(settings, f"  复用库 staging/ 里上一趟留下的 {len(removed)} 份残留删了")
        return run_with_ledger(settings)
    except (units8.RunCondition, judge.RunConditionError, reuse.RunCondition) as error:
        print(f"{units8.today()} ✗ 运行条件不满足（退 78）：{error}", flush=True)
        return units8.EXIT_RUN_CONDITION


# ── 自证：假仓与假树取 reuse_library.fake_world；假单元脚本照真脚本的取用协议（REUSE_ENTRIES 取用、REUSE_READINGS 记读数），
#    每一步之后跑一个假判法（取来的也跑）；逐行编排换成修订 8 自证那个假的 ──
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
  identity=${step/baseline-run2/baseline-run1}; identity=${identity/-run-2/-run-1}   # 同键的两趟（Z1、Z7）读数相同
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
        printf 'row_number\t0\nlane\t0\npackages_checked_rebuilt\tcrates/core\nsubstituted_from\t-\nfailed_attempts\t0\ncopy_ns\t%s\nstatus\tok\ndevice_exit\t0\n' "$RANDOM" > "$directory/row-0/row-result.tsv"
        lane="$WORK_ROOT/lane0"; mkdir -p "$lane/target/release" "$lane/tree"; cp -r "$REPO/$TREE/crates" "$lane/tree/"
        sed "s#$REPO/$TREE#$lane/tree#g" "$REPO/$TREE/target/release/e171_three_oracle_collusion_after_the_oracle_fixes.d" > "$lane/target/release/e171_three_oracle_collusion_after_the_oracle_fixes.d"
        roles+=("compiled=$lane") ;;
      *)
        echo "E7RESULT name=fake step=$identity salt=$salt" > "$product"
        printf 'E7TIMING name=stream_time stream=a elapsed_ns=1000000000\nE7TIMING name=process_wall seconds=1%s exit=0\n' "$RANDOM" > "$timing"
        mkdir -p "$directory/s"; echo "$identity" > "$directory/s/states.tsv" ;;
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
    library: str
    environment: dict


def fake_world_eleven(root: str) -> WorldEleven:
    base = reuse.fake_world(os.path.join(root, "world"))
    script = judge.write_text(os.path.join(root, "fake-reuse-unit.sh"), FAKE_REUSE_UNIT_SCRIPT)
    rows = judge.write_text(os.path.join(root, "fake-rows.sh"), units8.FAKE_ROWS)
    resources = judge.write_text(os.path.join(root, "resources.json"), json.dumps({"hosts": [{"number": 1, "read": True, "cpu": {"available_threads": 40},
                                                                                          "memory": {"available": 120 << 30, "slice_room": 60 << 30}}]}))
    environment = dict(os.environ, UNITS_R8_SELFTEST="1", UNITS_R11_SELFTEST="1", REUSE_SELFTEST="1", UNITS_R8_UNIT_SCRIPT=script, UNITS_R8_ROWS_ORCHESTRATOR=rows,
                       UNITS_R8_RESOURCES_COMMAND=f"cat {shlex.quote(resources)}", UNITS_R8_PREREQUISITES_COMMAND="true", UNITS_R8_POLL_SECONDS="0.02",
                       UNITS_R8_REQUERY_SECONDS="0", FAKE_LOG=os.path.join(root, "fake.log"), REUSE_SELFTEST_TOOLCHAIN="cargo fake\nrustc fake\n",
                       REUSE_SELFTEST_BUILD_ENVIRONMENT="<构建环境：假> 0", REUSE_LIBRARY_SCRIPT=os.path.join(HERE, "reuse_library.py"), PYTHONDONTWRITEBYTECODE="1")
    return WorldEleven(root, base.repo, os.path.join(root, "library"), environment)


def h31_of_this_layer() -> dict[str, str]:
    return {path: reuse.sha256_file(path) for path in (os.path.abspath(__file__), UNIT_SCRIPT, os.path.join(HERE, "reuse_library.py"))}


def pass_configs(world: WorldEleven, name: str, lookup: bool, h31: bool = True) -> str:
    """一遍的两份配置：修订 8 那一份（H16 各遍不同：日期、导出根、日志目录）与修订 11 那一份（库同一个）。交回修订 11 那一份的路径。"""
    revision_8 = {"dry_run": False, "tree": "scripts/experiment-apparatus/target/fake-tree", "H3": {"snapshot_digest": "f" * 64}, "H8": {"mutations_table_sha256": "e" * 64},
                  "H14": {"reverse_states": 72, "main_states": 1207959552}, "H7": {},
                  "H16": {"run_date": f"2026-10-{name}", "run_suffix": "-r8", "dumps_root": f"scripts/experiment-apparatus/target/e171-r1-dumps/fake-{name}",
                          "logs_directory": os.path.join(world.root, f"logs-{name}")}}
    revision_8_path = judge.write_text(os.path.join(world.root, f"revision-8-{name}.json"), json.dumps(revision_8))
    revision_11 = {"dry_run": False, "revision_8_config": revision_8_path, "H30": {"library_root": world.library, "lookup": lookup},
                   "H31": {"files": h31_of_this_layer() if h31 else None}}
    return judge.write_text(os.path.join(world.root, f"revision-11-{name}.json"), json.dumps(revision_11))


def run_pass(world: WorldEleven, config: str, extra: dict[str, str] | None = None) -> tuple[int, str]:
    completed = subprocess.run([sys.executable, "-B", os.path.abspath(__file__), "run", "--config", config, "--repo", world.repo], cwd=world.repo,
                               env=dict(world.environment, **(extra or {})), capture_output=True, text=True, timeout=300)
    return completed.returncode, completed.stdout + completed.stderr


def ledger_verdict(world: WorldEleven, name: str) -> str:
    path = os.path.join(world.repo, f"research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-{name}-s1-{LEDGER_STEM}-r8.out")
    return open(path, encoding="utf-8").read().strip().splitlines()[-1] if os.path.exists(path) else "账本不在"


def fake_log_lines(world: WorldEleven, start: int = 0) -> list[str]:
    path = world.environment["FAKE_LOG"]
    return open(path, encoding="utf-8").read().splitlines()[start:] if os.path.exists(path) else []


def cell_reuse_end_to_end(scratch: str) -> tuple[bool, str]:
    """第一遍查库关（全部现跑、入库）；第二遍查库开：除 step-9、step-10-run-2 外全部取来，每一步的假判法照跑，账本 reused=30；
    第三遍查库开、假判法在一个取来的单元（step-1）上判红：那个单元红、整趟退 1（取来的单元判法照跑，判红照停）。"""
    world = fake_world_eleven(scratch)
    first_code, first_output = run_pass(world, pass_configs(world, "11", False))
    first_lines = len(fake_log_lines(world))
    second_code, second_output = run_pass(world, pass_configs(world, "12", True))
    second = fake_log_lines(world, first_lines)
    judged = [line.split() for line in second if line.startswith("judge ")]
    fresh = sorted(words[2] for words in judged if words[3] != "reused")
    third_start = len(fake_log_lines(world))
    third_code, third_output = run_pass(world, pass_configs(world, "13", True), {"UNITS_R11_FAKE_JUDGE_RED": "s2-device-baseline-streams-1-2"})
    third = fake_log_lines(world, third_start)
    first_verdict, second_verdict = ledger_verdict(world, "11"), ledger_verdict(world, "12")
    good = (first_code == 0 and " reused=0 " in first_verdict and " units_cover_registered=true " in first_verdict and " check_mismatches=0 " in first_verdict
            and second_code == 0 and len(judged) == 32 and fresh == ["baseline-run2", "q5-main-before-the-first-barrier-of-call-3-run-2"]
            and " reused=30 " in second_verdict and " check_mismatches=0 " in second_verdict
            and third_code == 1 and "judge-red s2-device-baseline-streams-1-2" in third and any(line.startswith("judge step-1 ") and " reused " in line for line in third))
    return good, (f"第一遍退 {first_code}（{first_verdict[22:110]}）；第二遍退 {second_code}，判法跑了 {len(judged)} 步（要 32），现跑的 {fresh}，"
                  f"账本 {second_verdict[22:110]}；第三遍（取来的 step-1 判红）退 {third_code}")


def cell_h31_refused(scratch: str) -> tuple[bool, str]:
    world = fake_world_eleven(scratch)
    code, output = run_pass(world, pass_configs(world, "21", False, h31=False))
    return code == 78 and not fake_log_lines(world) and "H31" in output, f"H31 没填：退 {code}，假单元起过 {bool(fake_log_lines(world))}"


def cell_s18_stops(scratch: str) -> tuple[bool, str]:
    """同键第二次现跑、读数不同（假产物掺一个字）：RC1 present-differs → S18：库冻住、那个单元红、整趟退 1。"""
    world = fake_world_eleven(scratch)
    first_code, _ = run_pass(world, pass_configs(world, "31", False))
    second_code, output = run_pass(world, pass_configs(world, "32", False), {"UNITS_R11_FAKE_SALT_STEP": "s2-device-baseline-streams-1-2"})
    frozen = reuse.Library(world.library).frozen_reason()
    return first_code == 0 and second_code == 1 and bool(frozen) and "S18 触发" in output, \
        f"第一遍退 {first_code}；第二遍（step-1 掺一个字）退 {second_code}；库冻住 {bool(frozen)}"


STEP_1_VERDICT = ("E7RESULT name=verdict s1_failed_checks=0 s4_digest_mismatches=0 crash_facts_mask_mismatches=0 s3_o2_judgement_mismatches=0 "
                  "s9_failed_checks=0 streams_not_exported=0 o1_comparison_red_calls=0 o1_crash_judge_red_states=0 o1_reference_filesystem_red_states=0 "
                  "o2_red_states={o2} recovery_panic_states=0\n")


def real_script_step_1(scratch: str, label: str, world, library: reuse.Library, o2_red_states: int) -> tuple[int, list, str, str]:
    """真的 segment_one_unit_revision_11.sh 跑 step-1，编排给的条目里的产物 o2_red_states 取给定值。交回 (退出码, 记下的读数, 产物路径, 条目里那份产物)。"""
    source = os.path.join(scratch, f"reading-{label}")
    reading = {"product": judge.write_text(os.path.join(source, "product"), STEP_1_VERDICT.format(o2=o2_red_states)),
               "timing": judge.write_text(os.path.join(source, "timing"), "E7TIMING name=stream_time stream=a elapsed_ns=1000000000\n"
                                                                          "E7TIMING name=process_wall seconds=2.000 exit=0\n"),
               "directory": os.path.dirname(judge.write_text(os.path.join(source, "export", "s", "states.tsv"), "states\n"))}
    key = reuse.sha256_bytes(label.encode("utf-8"))
    library.insert(key, f"selftest {label}\n", "device", reading, {"exit": "0"})
    unit_directory = os.path.join(scratch, f"unit-{label}")
    os.makedirs(unit_directory)
    entries = judge.write_text(os.path.join(unit_directory, "reuse-entries.tsv"), f"s2-device-baseline-streams-1-2\t{library.entry_path(key)}\n")
    tree = os.path.relpath(world.tree, world.repo)
    environment = dict(os.environ, REPO=world.repo, RESEARCH=HERE, PREFIX=f"research/results/fake-{label}", SUFFIX="-r8", TAG="", DUMPS=f"scripts/experiment-apparatus/target/fake-dumps-{label}",
                       TREE=tree, BIN="/bin/false", E170_BIN="/bin/false", SNAPSHOT_DIGEST=judge.row_driver.snapshot_digest(world.tree)[0],
                       MUTATIONS_SHA256=reuse.sha256_file(os.path.join(world.tree, "crates/mutations.tsv")), CONFIG="-", THREADS="1", CAP_SMALL="4G",
                       CAP_LARGE="6G", WORK_ROOT=os.path.join(unit_directory, "work"), UNIT_LOG=os.path.join(unit_directory, "unit.log"),
                       UNIT_PRODUCTS=os.path.join(unit_directory, "products.tsv"), UNIT_DIRECTORIES=os.path.join(unit_directory, "directories.tsv"),
                       REUSE_LIBRARY=library.root, REUSE_ENTRIES=entries, REUSE_READINGS=os.path.join(unit_directory, "reuse-readings.tsv"), PRINT_ONLY="0")
    code = subprocess.run(["bash", UNIT_SCRIPT, "step-1"], cwd=world.repo, env=environment, capture_output=True, text=True, timeout=120).returncode
    product = os.path.join(world.repo, f"research/results/fake-{label}-s2-device-baseline-streams-1-2-r8.out")
    return code, reuse.readings_file_lines(environment["REUSE_READINGS"]), product, os.path.join(library.entry_path(key), "reading", "product")


def cell_unit_script_fetch(scratch: str) -> tuple[bool, str]:
    """真的单元脚本：编排给了条目就取用（硬链接、不起装置），记一行 reused，判法（check_product.py verdict、device_timing.py check）照跑；
    条目里的产物不合判法（o2_red_states=1）时单元红。"""
    world, library = reuse.fake_world(os.path.join(scratch, "world")), reuse.library_in(scratch)
    good_code, good_lines, good_product, good_entry = real_script_step_1(scratch, "good", world, library, 0)
    red_code, red_lines, _, _ = real_script_step_1(scratch, "red", world, library, 1)
    linked = os.path.exists(good_product) and os.stat(good_product).st_ino == os.stat(good_entry).st_ino
    states = [(step, state, roles.keys() and sorted(roles)) for step, state, _, roles in good_lines]
    good = good_code == 0 and linked and states == [("s2-device-baseline-streams-1-2", "reused", ["directory", "product", "timing"])] \
        and red_code == 1 and [state for _, state, _, _ in red_lines] == ["reused"]
    return good, f"合判法的条目：退 {good_code}，产物是条目的硬链接 {linked}，记下 {states}；不合判法的：退 {red_code}"


SELFTEST_CELLS = {"reuse-e2e": cell_reuse_end_to_end, "unit-script-fetch": cell_unit_script_fetch, "h31-refused": cell_h31_refused,
                  "s18-stops": cell_s18_stops}
SELFTEST_SWITCHES = {"h31-unchecked": "h31-refused", "s18-ignored": "s18-stops"}


def selftest(argument_list: list[str]) -> int:
    os.environ[SELFTEST_VARIABLE] = "1"
    return harness.run_selftest(os.path.abspath(__file__), argument_list, SELFTEST_CELLS, SELFTEST_SWITCHES, BREAK_VARIABLE, SELFTEST_VARIABLE)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
