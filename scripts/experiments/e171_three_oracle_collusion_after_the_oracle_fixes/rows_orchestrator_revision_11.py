#!/usr/bin/env python3
"""E171 第 1 次重跑·修订 11：修订 8 那一趟第 11–13 步的多机逐行编排，带复用（登记 research/prompts/e171-r1-prereg.md 修订 11 第 11.2–11.5、11.9、11.10 节）。
由 segment_one_units_revision_11.py 在修订 8 的逐行起跑闸都绿之后起（同修订 8 的命令行），修订 11 占位的路径从环境变量 E171_REVISION_11_CONFIG 取。

为什么另写一层：rows_orchestrator_revision_8.py（e6f1071e…）一个字节不改。这一层按路径载入它（开跑先核 sha256，不是就退 3；它照旧按 sha256 载入
第一段编排器 7bfbdb84… 与 cross_host_o3_check.py），修订 8 那一层改的几处（落点、H8 核对、跨机 o3 比法、停派文件、加槽）原样装上，再在第一段编排器上换三处：
  ① 第 11、12 步派发之前（phase_rows_and_checks）：每行照第 11.2、11.3 节算键（L 取 --baseline-timing 那份 step-7 计时的 process_wall），查库、写标记
    （<H16 导出根>/reuse-markers/segment-one-rows/），命中的行照第 11.5 节「取用」先写标记、再把条目里的行目录硬链接进行输出根（row-result.tsv 最后链），
    编排器原有的续跑因此当它做完了、不派；合成时由编排器原有的 --skip-finished 重报把它并进产物（合并规则 ①–⑦ 不改）。第 12 步 row-0 与跨机那一份 X
    （RC2）、要补位的行（按名字取不到、锚点不是恰 1 次、碰到模块集合）永远现跑；
  ② 一个分片结束（finish_shard）：编排器原来那样记退出码、收取回的行，再把这一分片现跑完的每一行照入库条件入库（库里有同键条目照 RC1 比，第二趟 RC2）；
    本机的行先照 RC4 核那一槽这一次编译的 dep-info（<工作根>/lane0/target/release/<装置>.d，相对 <工作根>/lane0/tree），不符不入；
    别的台取回的行入库之前另核三样（第 11.5 节「两台」）：行结果里 row_number 与 mutation_line_sha256 对得上键里 [mutation]、那一分片的 row_time_limit
    秒数等于键里的 L、那一分片的 snapshot_digest 等于 H3；不过不入、记一行。别的台的行不做 RC4（H35：peer_side.py 是修订 6 那一趟钉住的那一份，
    这一次没改，那边编完的 dep-info 不取回），只有第 11.2 节的静态拦。S18 触发就冻库、这一分片记没跑成（编排器不再派新行，在跑的跑完再停，退 1）；
  ③ 第 13 步（phase_timeout_reruns）：超时的行（限时再乘 3）照同一套查库、取用。
  收尾由标记现合复用账本 <H16 前缀>-reuse-ledger-rows<H16 后缀>.out（登记的行：第 11、12 步各行、多机时 X、第 13 步超时重跑的各行），N + M 不等于登记数退 1。
用法：
  rows_orchestrator_revision_11.py run  --config C --repo R --baseline-timing B --stop-file S      （同修订 8；E171_REVISION_11_CONFIG 要给）
  rows_orchestrator_revision_11.py plan --config C --repo R --baseline-timing B --stop-file S      （修订 8 的 plan 原样，另打每行的键与库里有没有；只读）
  rows_orchestrator_revision_11.py --selftest [--cells 格,…] [--rounds N]
退出码：同修订 8 逐行编排（0、1、2、3、78、143）；账本 N + M 不等于登记数退 1；E171_REVISION_11_CONFIG 没给、库目录写不了退 78。

弄坏开关 ROWS_R11_BREAK=<项>（逗号连几个；只在 --selftest 起的子进程里认，ROWS_R11_SELFTEST=1）。项名、弄坏哪一处、打开后红在哪一格：
  peer-checks-skipped    别的台取回的行入库之前不核那三样                 peer-checks
  s18-ignored            S18 触发了照样跑完、库不冻                       s18-stops
  sweep-skipped、temporary-kept（自证外壳 revision_8_selftest_harness.py 的两个）     no-leftover-processes、temporary-directory-removed
"""
from __future__ import annotations

import dataclasses
import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import types

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import device_timing  # noqa: E402
import frozen_tree_lines_revision_8 as judge  # noqa: E402
import reuse_library as reuse  # noqa: E402
import revision_8_selftest_harness as harness  # noqa: E402

BREAK_VARIABLE = "ROWS_R11_BREAK"
SELFTEST_VARIABLE = "ROWS_R11_SELFTEST"
CONFIG_VARIABLE = "E171_REVISION_11_CONFIG"
ROWS_R8_PATH = os.path.join(HERE, "rows_orchestrator_revision_8.py")
ROWS_R8_SHA256 = "e6f1071e72d06695c6638a9e6ebf214b0f867744b4da2e30a490be87afb9f444"
EXIT_WRONG_PINNED_FILE = 3
LEDGER_STEM = "reuse-ledger-rows"
MARKERS_NAME = "segment-one-rows"


def load_pinned(path: str, expected: str, module_name: str) -> types.ModuleType:
    actual = reuse.sha256_file(path)
    if actual != expected:
        raise RuntimeError(f"{path} 的 sha256 是 {actual}，不是 {expected}")
    specification = importlib.util.spec_from_file_location(module_name, path)
    module = importlib.util.module_from_spec(specification)
    sys.dont_write_bytecode = True
    sys.modules[specification.name] = module
    specification.loader.exec_module(module)
    return module


try:
    rows8 = load_pinned(ROWS_R8_PATH, ROWS_R8_SHA256, "rows_orchestrator_revision_8_e6f1071e")
except RuntimeError as wrong_file:
    print(f"✗ {wrong_file}（这一层调的是钉住的那一份的函数；改过就先核这一层，再改钉的 sha256）", file=sys.stderr)
    sys.exit(EXIT_WRONG_PINNED_FILE)
one = rows8.one


def broken(item: str) -> bool:
    return harness.broken(BREAK_VARIABLE, item)


@dataclasses.dataclass
class RowsReuse:
    """这一层的复用状态。decisions：(步, 行) → 决定；registered：账本登记的行名（第 11、12 步与 X 在派发之前登记，第 13 步的在它开头登记）。"""
    library: reuse.Library
    lookup: bool
    markers: str
    decisions: dict
    registered: list
    inputs: reuse.TreeInputs | None = None


STATE: dict = {"reuse": None}
ORIGINAL: dict = {}


def tree_inputs(context) -> reuse.TreeInputs:
    """算键用的那棵树（第一次用到时现算）；dep-info 里有树外路径就是 S18 ②：冻库、停（编排器原有的 Stopped，退 1）。"""
    state = STATE["reuse"]
    if state.inputs is None:
        try:
            state.inputs = reuse.tree_inputs_for_run(context.repo, context.tree, dict(os.environ))
        except reuse.ClosureOutsideTheTree as error:
            reuse.append_event(state.markers, f"E7RESULT name=reuse_closure_check unit=tree key=- matches=false outside-the-tree={reuse.value(str(error)[-120:])}")
            state.library.freeze(f"S18 ②：{error}")
            raise one.Stopped(f"S18 ②（闭包认不全）：{error}")
    return state.inputs


def row_limit(context, rerun: bool) -> int | None:
    """L：--baseline-timing 那份（修订 8 第 7 步）计时的 process_wall，max(1800, ⌈3T⌉)，第 13 步再乘 3；那份计时还不在交回 None（只有 plan 会碰上）。"""
    path = os.path.join(context.repo, context.baseline_timing)
    if not os.path.exists(path):
        return None
    baseline = device_timing.read_baseline_timing(open(path, encoding="utf-8").read().splitlines())
    return device_timing.row_time_limit_seconds(baseline.process_wall_seconds, rerun) if baseline.process_wall_seconds is not None else None


def row_keys(context, steps: tuple[str, ...], rows_of_step: dict | None = None) -> list[tuple[reuse.RowUnit, dict, str]]:
    inputs, selected = tree_inputs(context), reuse.selected_row_lines(context.repo)
    found = []
    for row in reuse.segment_one_row_units(steps, rows_of_step):
        sections, reason = reuse.row_sections(inputs, row, selected, reuse.limit_lines(row_limit(context, row.rerun)), [])
        found.append((row, sections, reason))
    return found


def decide_and_fetch(context, steps: tuple[str, ...], rows_of_step: dict | None = None) -> None:
    """派发之前：每行查库、写标记、取来命中的（第 11.5 节「与分片、续跑怎么叠」）；取来的行在 row-hosts.tsv 记一行「reused」（台号是原那一趟的）。"""
    state = STATE["reuse"]
    pairs, rows_of_name = [], {}
    for row, sections, reason in row_keys(context, steps, rows_of_step):
        decision, problems = reuse.decide(state.library, row.name, "driver-row", sections, reason, state.lookup)
        if problems:
            reuse.append_event(state.markers, f"E7RESULT name=reuse_corrupted unit={reuse.value(row.name)} key={decision.key} problems={reuse.value(','.join(problems[:6]))}")
        state.decisions[(row.owner, row.number)] = decision
        rows_of_name[row.name] = row
        if row.name not in state.registered:
            state.registered.append(row.name)
        pairs.append((decision, os.path.join(context.step_root(one.STEPS[row.owner]), f"row-{row.number}")))
    fetched, demoted = reuse.fetch_rows(state.library, state.markers, pairs)
    for decision in demoted:
        state.decisions[(rows_of_name[decision.unit].owner, rows_of_name[decision.unit].number)] = decision
    for name in fetched:
        row = rows_of_name[name]
        origin = state.decisions[(row.owner, row.number)].meta.get("origin_host", "1")
        one.append_ledger(context, row.owner, row.number, int(origin) if origin.isdigit() else 1, "reused")
    counts = {state_name: sum(1 for decision, _ in pairs if decision.state == state_name) for state_name in (reuse.STATE_REUSED, reuse.STATE_FRESH_ALWAYS)}
    one.say(context, f"  复用（{','.join(steps)}）：{len(pairs)} 行，库里取来 {len(fetched)} 行（命中 {counts[reuse.STATE_REUSED]}），永远现跑 "
                     f"{counts[reuse.STATE_FRESH_ALWAYS]} 行，取用时条目坏了改现跑 {len(demoted)} 行")


def phase_rows_with_reuse(context, hosts, allocations, setup, expected_hashes) -> None:
    decide_and_fetch(context, ("s11", "s12", "sx") if len(hosts) > 1 else ("s11", "s12"))
    ORIGINAL["phase_rows_and_checks"](context, hosts, allocations, setup, expected_hashes)


def phase_timeout_reruns_with_reuse(context, hosts, setup, expected_hashes) -> None:
    timed = {key: one.timed_out_rows(context.product(one.STEPS[source].stem)) for key, source in (("r11", "s11"), ("r12", "s12"))}
    rows_of_step = {key: rows for key, rows in timed.items() if rows and not os.path.exists(context.product(one.STEPS[key].stem))}
    if rows_of_step:
        decide_and_fetch(context, tuple(rows_of_step), rows_of_step)
    ORIGINAL["phase_timeout_reruns"](context, hosts, setup, expected_hashes)


def command_value(words: list[str], option: str) -> str:
    return words[words.index(option) + 1] if option in words and words.index(option) + 1 < len(words) else ""


def peer_check_problems(context, record: dict, row: int, row_directory: str, decision: reuse.Decision) -> list[str]:
    """别的台取回的行入库之前另核三样（reuse_library.peer_row_problems；快照摘要比发给别的台的那一份冻结树快照）。交回不过的那几样。"""
    if broken("peer-checks-skipped"):
        return []
    product = open(record["product"], encoding="utf-8").read() if os.path.exists(record["product"]) else ""
    return reuse.peer_row_problems(row_directory, row, decision.text, product, frozen_snapshot_digest(context))


def frozen_snapshot_digest(context) -> str:
    if "frozen_snapshot_digest" not in STATE:
        STATE["frozen_snapshot_digest"] = one.row_driver_module(context).snapshot_digest(os.path.join(context.shards, "frozen-snapshot"))[0]
    return STATE["frozen_snapshot_digest"]


def finish_shard_with_reuse(context, record: dict, code: int) -> bool:
    """编排器原来的 finish_shard，再把这一分片现跑完的行入库（本机的先 RC4，别的台的先核三样）；S18 触发就冻库、这一分片记没跑成。"""
    shard_ran_through = ORIGINAL["finish_shard"](context, record, code)
    state = STATE["reuse"]
    if record["kind"] not in ("local", "peer"):
        return shard_ran_through
    step = one.STEPS[record["step"]]
    for row in record["rows"]:
        decision = state.decisions.get((record["step"], row))
        row_directory = os.path.join(context.step_root(step), f"row-{row}")
        if decision is None or decision.state == reuse.STATE_REUSED or not os.path.exists(os.path.join(row_directory, "row-result.tsv")):
            continue
        if record["kind"] == "local":
            lane = os.path.join(command_value(record["command"], "--work-root"), "lane0")
            if not reuse.closure_check(state.markers, decision, os.path.join(lane, "tree"), os.path.join(lane, "target/release", reuse.DEVICE_BINARY + ".d"),
                                       reuse.mutated_closure_paths(tree_inputs(context))):
                continue
            insertable = reuse.row_insertable(row_directory)
        else:
            problems = peer_check_problems(context, record, row, row_directory, decision)
            insertable = (False, "peer-check-" + ",".join(problems)) if problems else reuse.row_insertable(row_directory)
        fields = reuse.row_result_fields(row_directory)
        meta = {"exit": fields.get("device_exit", "-"), "origin_run": f"{context.prefix}{context.tag}{context.suffix}", "origin_product": step.stem,
                "origin_host": str(record["host"]), "threads": command_value(record["command"], "--threads"),
                "closure_check": "rc4" if record["kind"] == "local" else "static-only"}
        outcome = reuse.record_finished(state.library, state.markers, decision, {"row": row_directory}, insertable, meta)
        one.say(context, f"  入库 {decision.unit}（{one.host_name(record['host'])}）：{outcome}（{insertable[1] or '入库条件过'}）")
    problems = reuse.s18_problems(state.markers)
    if problems and not broken("s18-ignored"):
        state.library.freeze(f"S18（修订 11 第 11.9 节）：{problems[0][:300]}")
        one.say(context, f"✗ S18 触发（{len(problems)} 行事件，第一行：{problems[0][:200]}）：库冻住、不再取不再入，这一分片记没跑成，交主 agent")
        return False
    return shard_ran_through


def plan_lines(context) -> list[str]:
    """plan：每行的键与库里有没有（只读：不建库、不写标记、不挪坏条目）。"""
    state = STATE["reuse"]
    lines = [f"── 复用（修订 11）── 库 {state.library.root}（{'在' if os.path.isdir(state.library.root) else '还没建'}；冻住 {state.library.frozen_reason() or '否'}）；"
             f"查库 {'开' if state.lookup else '关'}；标记 {state.markers}"]
    if row_limit(context, False) is None:
        return lines + [f"限时要的那份计时 {context.baseline_timing} 还不在（修订 8 第 7 步没绿）：各行的键到起跑时现算"]
    for row, sections, reason in row_keys(context, ("s11", "s12")):
        key = reuse.key_of(sections)
        present = os.path.isdir(state.library.entry_path(key))
        lines.append(f"行 {row.name}：键 {key[:16]}；{('永远现跑（' + reason + '）') if reason else ('库里有同键条目' if present else '库里没有')}")
    return lines


def run_with_ledger(context, plan_only: bool) -> int:
    """编排器原来的 run；收尾现合复用账本（跑完装进 research/results，没跑完只写日志目录）。plan 另打每行的键。"""
    if plan_only:
        code = ORIGINAL["run"](context, True)
        for line in plan_lines(context):
            print(line)
        return code
    state = STATE["reuse"]
    code = None
    try:
        code = ORIGINAL["run"](context, False)
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


def install(reuse_config: dict, repo: str, dumps: str) -> None:
    h30 = reuse_config.get("H30") or {}
    STATE["reuse"] = RowsReuse(reuse.Library(h30.get("library_root") or reuse.DEFAULT_LIBRARY_ROOT), bool(h30.get("lookup", True)),
                               os.path.join(repo, dumps, "reuse-markers", MARKERS_NAME), {}, [])
    for name in ("phase_rows_and_checks", "finish_shard", "phase_timeout_reruns", "run"):
        ORIGINAL[name] = getattr(one, name)
    one.phase_rows_and_checks = phase_rows_with_reuse
    one.finish_shard = finish_shard_with_reuse
    one.phase_timeout_reruns = phase_timeout_reruns_with_reuse
    one.run = run_with_ledger


def main(argument_list: list[str]) -> int:
    for variable, selftest_variable in ((BREAK_VARIABLE, SELFTEST_VARIABLE), (reuse.BREAK_VARIABLE, reuse.SELFTEST_VARIABLE)):
        refusal = harness.refuse_break_outside_selftest(variable, selftest_variable)
        if refusal:
            print(refusal, file=sys.stderr)
            return 2
    if argument_list[:1] == ["--selftest"]:
        return selftest(argument_list[1:])
    config_path = os.environ.get(CONFIG_VARIABLE, "")
    repo = os.path.abspath(argument_list[argument_list.index("--repo") + 1] if "--repo" in argument_list else os.getcwd())
    if not config_path or not os.path.exists(config_path):
        print(f"✗ 修订 11 占位要经 {CONFIG_VARIABLE} 给（由 segment_one_units_revision_11.py 起时设好）：{config_path or '没给'}（退 78）", file=sys.stderr)
        return one.EXIT_RUN_CONDITION
    reuse_config = json.load(open(config_path, encoding="utf-8"))
    mode = argument_list[0] if argument_list else ""
    if mode == "run":
        problems = reuse.h31_problems(repo, reuse_config)
        try:
            if problems:
                raise reuse.RunCondition("H31 不过：" + "；".join(problems[:4]))
            reuse.Library((reuse_config.get("H30") or {}).get("library_root") or reuse.DEFAULT_LIBRARY_ROOT).check_run_condition(repo)
        except reuse.RunCondition as error:
            print(f"✗ 运行条件不满足（退 78）：{error}", file=sys.stderr)
            return one.EXIT_RUN_CONDITION
    original_configure = rows8.configure

    def configure_with_reuse(config: dict, stop_file: str) -> None:
        original_configure(config, stop_file)
        install(reuse_config, repo, str(judge.section(config, "H16").get("dumps_root")))

    rows8.configure = configure_with_reuse
    return rows8.main(argument_list)


# ── 自证：借第一段编排器的假世界（假驱动、假 multi-host-run、假机器清单与资源；这里只用本机一台），假树补上算键要的几样；
#    假驱动外面包一层，在 <工作根>/lane0 下放一份「这一次编译」的 dep-info 给 RC4 ──
FAKE_RUN_DATE = "2026-10-09"
FAKE_DRIVER = os.path.join(HERE, "fake_row_driver.py")


@dataclasses.dataclass
class WorldEleven:
    root: str
    base: object
    library: str
    environment: dict


def fake_world_eleven(root: str) -> WorldEleven:
    base = one.fake_world(os.path.join(root, "world"), hosts_code=3)
    tree = os.path.join(base.repo, "scripts/experiments/target/fake-frozen-tree")
    reuse.furnish_rows_world(base.repo, tree, tuple(row for rows in one.STEP_ROWS.values() for row in rows))
    wrapper = judge.write_text(os.path.join(root, "fake-driver-with-dep-info.py"), reuse.FAKE_DRIVER_WRAPPER.replace("FAKE_DRIVER_PATH", FAKE_DRIVER))
    environment = dict(base.environment, ROWS_R8_SELFTEST="1", ROWS_R11_SELFTEST="1", REUSE_SELFTEST="1", ROWS_ORCHESTRATOR_DRIVER=wrapper,
                       REUSE_SELFTEST_TOOLCHAIN="cargo fake\nrustc fake\n", REUSE_SELFTEST_BUILD_ENVIRONMENT="<构建环境：假> 0")
    return WorldEleven(root, base, os.path.join(root, "library"), environment)


def pass_configs(world: WorldEleven, name: str, lookup: bool) -> tuple[str, str, str]:
    """一遍的修订 8 配置（H16 各遍不同；H3、H8 照补好的假树现算）、修订 11 配置（库同一个）、这一遍 step-7 计时（各遍同一个 T）。"""
    tree = os.path.join(world.base.repo, "scripts/experiments/target/fake-frozen-tree")
    run_date = f"{FAKE_RUN_DATE[:-2]}{name}"
    config = {"tree": "scripts/experiments/target/fake-frozen-tree", "H3": {"snapshot_digest": judge.row_driver.snapshot_digest(tree)[0]},
              "H8": {"mutations_table_sha256": judge.sha256_of_file(os.path.join(tree, judge.MUTATIONS_TABLE))},
              "H16": {"run_date": run_date, "run_suffix": "-r8", "dumps_root": f"scripts/experiments/target/fake-dumps-{name}", "logs_directory": os.path.join(world.root, f"logs-{name}")},
              "H18": {"peer_kit": "research/e171-s1-final-peer-kit", "peer_snapshot": "research/e171-s1-final-frozen-snapshot", "peer_out": "scripts/experiments/target/e171-s1-final-peer-out"}}
    prefix = os.path.join(world.base.repo, rows8.placeholders.product_prefix(run_date))
    for stem, seconds in (("baseline-run1", "100.500"), ("positive-control-o1-crash-judge", "80.000"), ("positive-control-reference-filesystem", "70.000")):
        judge.write_text(f"{prefix}-{stem}-r8-timing.out", f"E7TIMING name=stream_time stream=a elapsed_ns=1000000000\nE7TIMING name=process_wall seconds={seconds} exit=0\n")
    revision_8 = judge.write_text(os.path.join(world.root, f"revision-8-{name}.json"), json.dumps(config))
    h31 = {path: reuse.sha256_file(path) for path in (os.path.abspath(__file__), os.path.join(HERE, "reuse_library.py"))}
    revision_11 = judge.write_text(os.path.join(world.root, f"revision-11-{name}.json"),
                                   json.dumps({"H30": {"library_root": world.library, "lookup": lookup}, "H31": {"files": h31}}))
    return revision_8, revision_11, rows8.placeholders.product_prefix(run_date) + "-baseline-run1-r8-timing.out"


def run_pass(world: WorldEleven, name: str, lookup: bool, extra: dict[str, str] | None = None) -> tuple[int, str]:
    revision_8, revision_11, timing = pass_configs(world, name, lookup)
    command = [sys.executable, "-B", os.path.abspath(__file__), "run", "--config", revision_8, "--repo", world.base.repo, "--baseline-timing", timing,
               "--stop-file", os.path.join(world.root, f"stop-{name}")]
    completed = subprocess.run(command, cwd=world.base.repo, env=dict(world.environment, E171_REVISION_11_CONFIG=revision_11, **(extra or {})),
                               capture_output=True, text=True, timeout=300)
    return completed.returncode, completed.stdout + completed.stderr


def product_of(world: WorldEleven, name: str, stem: str) -> str:
    return os.path.join(world.base.repo, f"{rows8.placeholders.product_prefix(FAKE_RUN_DATE[:-2] + name)}-{stem}-r8.out")


def ran_rows(world: WorldEleven) -> list[str]:
    """假驱动真跑过（不是 --skip-finished 跳过）的行：driver.log 里 ran 开头的那几行。"""
    path = os.path.join(world.base.root, "driver.log")
    return [line.strip() for line in open(path, encoding="utf-8") if line.startswith("ran ")] if os.path.exists(path) else []


def merged_rows_without_timing(path: str) -> list[str]:
    return sorted(one.row_line_without_timing(line.rstrip("\n")) for line in open(path, encoding="utf-8") if line.startswith("row ")) if os.path.exists(path) else []


def cell_reuse_end_to_end(scratch: str) -> tuple[bool, str]:
    """第一遍查库关（16 行全部现跑、RC4 过、入库）；第二遍查库开：除第 12 步 row-0（RC2）外全部取来、不派，假驱动只真跑 row-0；
    合成的两份产物去耗时列之后与第一遍逐行相同（判法照跑、判过）；账本 reused=15、check_mismatches=0。"""
    world = fake_world_eleven(scratch)
    first_code, first_output = run_pass(world, "11", False)
    first_ran = len(ran_rows(world))
    second_code, second_output = run_pass(world, "12", True)
    second_ran = ran_rows(world)[first_ran:]
    ledger = open(product_of(world, "12", LEDGER_STEM), encoding="utf-8").read().strip().splitlines()[-1] if os.path.exists(product_of(world, "12", LEDGER_STEM)) else "账本不在"
    same_products = all(merged_rows_without_timing(product_of(world, "11", stem)) == merged_rows_without_timing(product_of(world, "12", stem))
                        and merged_rows_without_timing(product_of(world, "12", stem)) for stem in ("rows-driver", "rows-probe-driver"))
    closure_events = sum(1 for line in open(os.path.join(world.base.repo, "scripts/experiments/target/fake-dumps-11/reuse-markers", MARKERS_NAME, "events.tsv"), encoding="utf-8")
                         if "name=reuse_closure_check" in line and "matches=true" in line) if first_code == 0 else 0
    good = (first_code == 0 and first_ran == 16 and closure_events == 16 and second_code == 0 and [line.split()[1] for line in second_ran] == ["row=0"]
            and same_products and " reused=15 " in ledger and " units_cover_registered=true " in ledger and " check_mismatches=0 " in ledger)
    return good, (f"第一遍退 {first_code}、真跑 {first_ran} 行、RC4 相符 {closure_events} 次；第二遍退 {second_code}、真跑 {[line.split()[1] for line in second_ran]}；"
                  f"两遍合成产物去耗时列相同 {same_products}；账本 {ledger[22:120]}；末行 {(second_output.strip().splitlines() or [''])[-1][:80]}")


def cell_peer_checks(scratch: str) -> tuple[bool, str]:
    """别的台取回的行入库之前的三样：都对得上交回空表；row_number、mutation_line_sha256、row_time_limit、snapshot_digest 各错一样各报一样。"""
    raw = "m-row-two\tcrates/core/src/other.rs\talpha\tbeta\t-p core\tt_two"
    decision = reuse.Decision("seg1-s12-row-15", "driver-row", "k", f"[mutation]\nrow 15\nmutation {raw}\n[limit]\nseconds 1800\n[python]\nx\n", reuse.STATE_FRESH, "no-entry")
    STATE["frozen_snapshot_digest"] = "d" * 64
    context = types.SimpleNamespace(shards=scratch)
    outcomes = {}
    for case, row_number, digest, limit, snapshot in (("good", "15", hashlib.sha256(raw.encode()).hexdigest(), "1800", "d" * 64),
                                                      ("row_number", "16", hashlib.sha256(raw.encode()).hexdigest(), "1800", "d" * 64),
                                                      ("mutation_line_sha256", "15", "0" * 64, "1800", "d" * 64),
                                                      ("row_time_limit", "15", hashlib.sha256(raw.encode()).hexdigest(), "2400", "d" * 64),
                                                      ("snapshot_digest", "15", hashlib.sha256(raw.encode()).hexdigest(), "1800", "e" * 64)):
        row_directory = os.path.join(scratch, case, "row-15")
        judge.write_text(os.path.join(row_directory, "row-result.tsv"), f"row_number\t{row_number}\nmutation_line_sha256\t{digest}\n")
        product = judge.write_text(os.path.join(scratch, case, "product.out"), f"name=row_time_limit seconds={limit} baseline_seconds=1\nsnapshot_digest={snapshot} files=3\n")
        outcomes[case] = peer_check_problems(context, {"product": product}, 15, row_directory, decision)
    good = outcomes["good"] == [] and all(outcomes[case] == [case] for case in outcomes if case != "good")
    return good, f"各样：{outcomes}"


def cell_s18_stops(scratch: str) -> tuple[bool, str]:
    """第二遍查库关、本机的假驱动往 device.out 的 E7RESULT 里掺一个字：同键入库时 RC1 present-differs → S18：库冻住、那一分片记没跑成、退 1。"""
    world = fake_world_eleven(scratch)
    first_code, _ = run_pass(world, "21", False)
    second_code, output = run_pass(world, "22", False, {"FAKE_DRIVER_SALT_HOST": "1", "FAKE_DRIVER_SALT_PART": "device"})
    frozen = reuse.Library(world.library).frozen_reason()
    return first_code == 0 and second_code == 1 and bool(frozen) and "S18 触发" in output, f"第一遍退 {first_code}；第二遍（掺字）退 {second_code}；库冻住 {bool(frozen)}"


SELFTEST_CELLS = {"reuse-e2e": cell_reuse_end_to_end, "peer-checks": cell_peer_checks, "s18-stops": cell_s18_stops}
SELFTEST_SWITCHES = {"peer-checks-skipped": "peer-checks", "s18-ignored": "s18-stops"}


def selftest(argument_list: list[str]) -> int:
    os.environ[SELFTEST_VARIABLE] = "1"
    return harness.run_selftest(os.path.abspath(__file__), argument_list, SELFTEST_CELLS, SELFTEST_SWITCHES, BREAK_VARIABLE, SELFTEST_VARIABLE)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
