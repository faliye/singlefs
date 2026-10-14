#!/usr/bin/env python3
"""E171 第 1 次重跑·修订 11：修订 8 那一趟第二段整表的多机编排，带复用（登记 research/prompts/e171-r1-prereg.md 修订 11 第 11.2–11.5、11.9、11.10 节）。
由 run_segment_two_revision_11_multi_host.sh 起，在仓根跑；修订 11 占位（revision_11_placeholders.json 那一形）点名修订 10 那一份占位（revision_10_config）。

为什么另写一层：rows_orchestrator_segment_two_revision_10.py（627e48c6…）一个字节不改。这一层按路径载入它（开跑先核 sha256，不是就退 3；它照旧按 sha256
载入 rows_orchestrator_segment_two.py 7d7b458b… 与第一段编排器 1dcfb8d0…），修订 10 那一层的几处（树与 T₂、H8 核对、落点、起跑闸、本机闸、停派文件、让路、
输入清单、S17 联动）由它自己的 run_layer 原样装上；这一层只在第二段编排器上再换四处：
  ① 整表派发之前（phase_rows）：281 行（编排器现算的行集）照第 11.2、11.3 节算键（L 取 T₂ 那份计时的 process_wall），查库、写标记
    （<H23 导出根>/reuse-markers/segment-two-rows/）；取来的行数 M ≥ 1 时照 RC3 挑 k = max(3, ⌈0.02 × M⌉) 行现跑（按 sha256(H23 后缀 ‖ 键) 从小到大，
    挑中的与挑法记一行 name=reuse_sample，即 H32），其余命中的照第 11.5 节「取用」硬链接进行输出根（row-result.tsv 最后链），编排器原有的续跑当它做完了、
    不派，合成时由原有的 --skip-finished 重报并进产物。跨机对照那一行（编排器现算的那一行）在整表与跨机那一次都永远现跑（RC2）；要补位的行（按名字取不到、
    锚点不是恰 1 次、碰到模块集合）永远现跑、不进库，补位协调照编排器原样；
  ② 一个分片结束（finish_shard_two）：编排器原来那样收尾，再把这一分片现跑完的行照入库条件入库（库里有同键条目照 RC1 比，第二趟 RC2，抽到的 RC3）；
    本机的行先照 RC4 核那一槽这一次编译的 dep-info，别的台取回的行先核三样（reuse_library.peer_row_problems；别的台不做 RC4，H35）；
    S18 触发就冻库、这一分片记没跑成（编排器不再派新行，在跑的跑完再停，退 1）；
  ③ 跨机对照（phase_cross_host）之前给那一行的跨机那一份登记、写标记（永远现跑）；
  ④ 超时重跑（phase_timeout_reruns，只另出产物）：超时的行（限时再乘 3）照同一套查库、取用。
  收尾由标记现合复用账本 <H23 前缀>-reuse-ledger-rows<H23 后缀>.out（登记的行：整表各行、多机时跨机那一份、超时重跑的各行），N + M 不等于登记数退 1。
用法：
  rows_orchestrator_segment_two_revision_11.py run  --config <修订 11 占位> [--repo R]
  rows_orchestrator_segment_two_revision_11.py plan --config <修订 11 占位> [--repo R]   修订 10 的 plan 原样，先打每行永远现跑的缘由与库在不在（只读）
  rows_orchestrator_segment_two_revision_11.py --selftest [--cells 格,…] [--rounds N]
退出码：同修订 10 第二段编排层（0、1、2、3、78、143）；账本 N + M 不等于登记数退 1；H31 不过、库目录写不了退 78。

弄坏开关 SEGMENT_TWO_R11_BREAK=<项>（逗号连几个；只在 --selftest 起的子进程里认，SEGMENT_TWO_R11_SELFTEST=1）。项名、弄坏哪一处、打开后红在哪一格：
  sample-skipped         取来的行一行都不挑出来现跑（RC3 不做）              reuse-e2e
  peer-checks-skipped    别的台取回的行入库之前不核那三样                    peer-checks
  s18-ignored            S18 触发了照样跑完、库不冻                          s18-stops
  sweep-skipped、temporary-kept（自证外壳 revision_8_selftest_harness.py 的两个）     no-leftover-processes、temporary-directory-removed
"""
from __future__ import annotations

import dataclasses
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

BREAK_VARIABLE = "SEGMENT_TWO_R11_BREAK"
SELFTEST_VARIABLE = "SEGMENT_TWO_R11_SELFTEST"
SEGMENT_TWO_R10_PATH = os.path.join(HERE, "rows_orchestrator_segment_two_revision_10.py")
SEGMENT_TWO_R10_SHA256 = "627e48c6f35e872f2eaf7be08f8c19ff4fead2d39e46bed23710f678108e2ace"
EXIT_WRONG_PINNED_FILE = 3
LEDGER_STEM = "reuse-ledger-rows"
MARKERS_NAME = "segment-two-rows"


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
    layer10 = load_pinned(SEGMENT_TWO_R10_PATH, SEGMENT_TWO_R10_SHA256, "rows_orchestrator_segment_two_revision_10_627e48c6")
except RuntimeError as wrong_file:
    print(f"✗ {wrong_file}（这一层调的是钉住的那一份的函数；改过就先核这一层，再改钉的 sha256）", file=sys.stderr)
    sys.exit(EXIT_WRONG_PINNED_FILE)
two = layer10.two
one = two.one


def broken(item: str) -> bool:
    return harness.broken(BREAK_VARIABLE, item)


@dataclasses.dataclass
class SegmentTwoReuse:
    """这一层的复用状态。decisions：(步, 行) → 决定（步是 rows、cross、rerun）；registered：账本登记的行名。"""
    library: reuse.Library
    lookup: bool
    markers: str
    suffix: str
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


def decide_and_fetch(context, step: str, rows: tuple[int, ...], sample: bool) -> None:
    """派发之前：每行查库；整表那一次照 RC3 挑 k 行改成现跑（fresh-check）；写标记、取来其余命中的；取来的行在 row-hosts.tsv 记一行「reused」。"""
    state = STATE["reuse"]
    inputs, selected = tree_inputs(context), reuse.selected_row_lines(context.repo)
    decided = []
    for row in reuse.segment_two_row_units(rows, context.cross_host_row, (step,)):
        sections, reason = reuse.row_sections(inputs, row, selected, reuse.limit_lines(row_limit(context, row.rerun)), [])
        decision, problems = reuse.decide(state.library, row.name, "driver-row", sections, reason, state.lookup)
        if problems:
            reuse.append_event(state.markers, f"E7RESULT name=reuse_corrupted unit={reuse.value(row.name)} key={decision.key} problems={reuse.value(','.join(problems[:6]))}")
        decided.append((row, decision))
    if sample and not broken("sample-skipped"):
        chosen = earlier_sample(state.markers)
        if chosen is None:
            chosen = reuse.rows_sample([decision for _, decision in decided], state.suffix)
            if chosen:
                reuse.append_event(state.markers, f"E7RESULT name=reuse_sample k={len(chosen)} rows={reuse.value(','.join(chosen))} "
                                                  f"taken_before_sampling={sum(1 for _, decision in decided if decision.state == reuse.STATE_REUSED)} suffix={state.suffix}")
        decided = [(row, dataclasses.replace(decision, state=reuse.STATE_FRESH_CHECK, reason="sample") if decision.unit in chosen and decision.state == reuse.STATE_REUSED
                    else decision) for row, decision in decided]
    pairs = []
    for row, decision in decided:
        state.decisions[(step, row.number)] = decision
        if row.name not in state.registered:
            state.registered.append(row.name)
        pairs.append((decision, os.path.join(context.step_root(two.step_of(context, step)), f"row-{row.number}")))
    fetched, demoted = reuse.fetch_rows(state.library, state.markers, pairs)
    numbers = {row.name: row.number for row, _ in decided}
    for decision in demoted:
        state.decisions[(step, numbers[decision.unit])] = decision
    for name in fetched:
        origin = state.decisions[(step, numbers[name])].meta.get("origin_host", "1")
        one.append_ledger(context, step, numbers[name], int(origin) if origin.isdigit() else 1, "reused")
    one.say(context, f"  复用（{step}）：{len(pairs)} 行，库里取来 {len(fetched)} 行，RC3 抽出来现跑 {sum(1 for _, decision in decided if decision.state == reuse.STATE_FRESH_CHECK)} 行，"
                     f"永远现跑 {sum(1 for _, decision in decided if decision.state == reuse.STATE_FRESH_ALWAYS)} 行，取用时条目坏了改现跑 {len(demoted)} 行")


def phase_rows_with_reuse(context, hosts, slots, setup, expected_hashes) -> None:
    if not os.path.exists(context.product(two.ROWS_STEP.stem)):
        decide_and_fetch(context, "rows", context.rows, True)
    ORIGINAL["phase_rows"](context, hosts, slots, setup, expected_hashes)


def phase_cross_host_with_reuse(context, hosts, setup, expected_hashes) -> None:
    if len(hosts) > 1 and not os.path.exists(context.product(two.cross_host_step(context.cross_host_row).stem)):
        decide_and_fetch(context, "cross", (context.cross_host_row,), False)
    ORIGINAL["phase_cross_host"](context, hosts, setup, expected_hashes)


def phase_timeout_reruns_with_reuse(context, hosts, setup, expected_hashes) -> None:
    timed = one.timed_out_rows(context.product(two.ROWS_STEP.stem))
    if timed and not os.path.exists(context.product(two.RERUN_STEP.stem)):
        decide_and_fetch(context, "rerun", timed, False)
    ORIGINAL["phase_timeout_reruns"](context, hosts, setup, expected_hashes)


def command_value(words: list[str], option: str) -> str:
    return words[words.index(option) + 1] if option in words and words.index(option) + 1 < len(words) else ""


def frozen_snapshot_digest(context) -> str:
    if "frozen_snapshot_digest" not in STATE:
        STATE["frozen_snapshot_digest"] = one.row_driver_module(context).snapshot_digest(os.path.join(context.shards, "frozen-snapshot"))[0]
    return STATE["frozen_snapshot_digest"]


def peer_check_problems(context, record: dict, row: int, row_directory: str, decision: reuse.Decision) -> list[str]:
    if broken("peer-checks-skipped"):
        return []
    product = open(record["product"], encoding="utf-8").read() if os.path.exists(record["product"]) else ""
    return reuse.peer_row_problems(row_directory, row, decision.text, product, frozen_snapshot_digest(context))


def finish_shard_with_reuse(context, record: dict, code: int) -> bool:
    """编排器原来的 finish_shard_two，再把这一分片现跑完的行入库（本机的先 RC4，别的台的先核三样）；S18 触发就冻库、这一分片记没跑成。"""
    shard_ran_through = ORIGINAL["finish_shard_two"](context, record, code)
    state = STATE["reuse"]
    if record["kind"] not in ("local", "peer"):
        return shard_ran_through
    step = two.step_of(context, record["step"])
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
        meta = {"exit": fields.get("device_exit", "-"), "origin_run": f"{context.prefix}{context.suffix}", "origin_product": step.stem,
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
    """plan：整表每行的键与库里有没有、永远现跑的缘由（只读：不建库、不写标记、不挪坏条目）。"""
    state = STATE["reuse"]
    lines = [f"── 复用（修订 11）── 库 {state.library.root}（{'在' if os.path.isdir(state.library.root) else '还没建'}；冻住 {state.library.frozen_reason() or '否'}）；"
             f"查库 {'开' if state.lookup else '关'}；标记 {state.markers}；RC3 挑法 sha256({state.suffix} ‖ 键)"]
    inputs, selected, limit = tree_inputs(context), reuse.selected_row_lines(context.repo), row_limit(context, False)
    counts: dict[str, int] = {}
    for row in reuse.segment_two_row_units(context.rows, context.cross_host_row):
        sections, reason = reuse.row_sections(inputs, row, selected, reuse.limit_lines(limit), [])
        key = reuse.key_of(sections)
        verdict = f"永远现跑（{reason}）" if reason else ("库里有同键条目" if os.path.isdir(state.library.entry_path(key)) else "库里没有")
        counts[verdict] = counts.get(verdict, 0) + 1
        lines.append(f"行 {row.name}：键 {key[:16]}；{verdict}")
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


def install(reuse_config: dict, repo: str, segment_two) -> None:
    h30 = reuse_config.get("H30") or {}
    STATE["reuse"] = SegmentTwoReuse(reuse.Library(h30.get("library_root") or reuse.DEFAULT_LIBRARY_ROOT), bool(h30.get("lookup", True)),
                                     os.path.join(repo, segment_two.dumps, "reuse-markers", MARKERS_NAME), segment_two.suffix, {}, [])
    for name in ("phase_rows", "finish_shard_two", "phase_cross_host", "phase_timeout_reruns", "run_two"):
        ORIGINAL[name] = getattr(two, name)
    two.phase_rows = phase_rows_with_reuse
    two.finish_shard_two = finish_shard_with_reuse
    two.phase_cross_host = phase_cross_host_with_reuse
    two.phase_timeout_reruns = phase_timeout_reruns_with_reuse
    two.run_two = run_two_with_ledger


def main(argument_list: list[str]) -> int:
    for variable, selftest_variable in ((BREAK_VARIABLE, SELFTEST_VARIABLE), (reuse.BREAK_VARIABLE, reuse.SELFTEST_VARIABLE),
                                        (layer10.BREAK_VARIABLE, layer10.SELFTEST_VARIABLE), (two.BREAK_VARIABLE, two.SELFTEST_VARIABLE),
                                        ("ROWS_ORCHESTRATOR_BREAK", "ROWS_ORCHESTRATOR_SELFTEST")):
        refusal = harness.refuse_break_outside_selftest(variable, selftest_variable)
        if refusal:
            print(refusal, file=sys.stderr)
            return 2
    if os.environ.get(two.SELFTEST_ROWS_VARIABLE) and os.environ.get(two.SELFTEST_VARIABLE) != "1":
        print(f"✗ {two.SELFTEST_ROWS_VARIABLE} 只在自证起的子进程里认（平常跑整表 1–281）；设了它就拒（退 2）", file=sys.stderr)
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
            library = reuse.Library((reuse_config.get("H30") or {}).get("library_root") or reuse.DEFAULT_LIBRARY_ROOT)
            library.check_run_condition(repo)
            library.clear_stale_staging()
        else:
            print(f"修订 11 占位 {os.path.abspath(arguments.config)}（dry_run={str(bool(reuse_config.get('dry_run'))).lower()}；{reuse_config.get('dry_run_note') or '-'}）；"
                  f"H31 {'；'.join(reuse.h31_problems(repo, reuse_config)) or '与现在的文件逐个相同'}；修订 10 占位 {revision_10_path}")
    except (reuse.RunCondition, judge.RunConditionError, KeyError, OSError) as error:
        print(f"✗ 运行条件不满足（退 78）：{error}", flush=True)
        print('     → 怎么办：照上面那句补齐运行条件（配置、冻结树、设备文件）再用同一条命令起', flush=True)
        return one.EXIT_RUN_CONDITION
    install(reuse_config, repo, segment_two)
    if arguments.mode == "plan":
        for line in static_plan_lines(repo, judge.tree_of(revision_8_config)):
            print(line)
    return layer10.run_layer(repo, revision_10_path, arguments.mode == "plan")


def static_plan_lines(repo: str, tree: str) -> list[str]:
    """plan 在交编排层之前先打的那几行（编排层在 T₂ 取不到时不起 run_two）：整表每行永远现跑与否（与 L 无关），键等 T₂ 定了现算。"""
    try:
        inputs = reuse.tree_inputs_for_run(repo, tree, dict(os.environ))
    except (reuse.ClosureOutsideTheTree, OSError, RuntimeError) as error:
        return [f"✗ 算键用的树 {tree} 认不全或读不到：{error}"]
    selected, reasons = reuse.selected_row_lines(repo), {}
    for row in reuse.segment_two_row_units(tuple(range(1, 282)), None):
        reason = reuse.row_sections(inputs, row, selected, reuse.limit_lines(None), [])[1] or "照常查库"
        reasons.setdefault(reason, []).append(row.number)
    return [f"复用·树 {tree}：装置 bin 的 dep-info {len(inputs.closure_files['device'])} 个文件；整表 281 行里 " +
            "；".join(f"{reason} {len(rows)} 行" + (f"（第 {','.join(map(str, rows[:8]))} 行）" if reason != "照常查库" else "") for reason, rows in sorted(reasons.items())) +
            "；各行的键要 L，T₂ 定了之后由编排层现算（跨机对照那一行另记永远现跑）"]


# ── 自证：借第二段编排器的假世界（假驱动、假 multi-host-run、假机器清单与资源），假树补上算键要的几样，配置照修订 10 自证那一形逐遍另写；
#    假驱动外面包一层，在 <工作根>/lane0 下放一份「这一次编译」的 dep-info 给 RC4 ──
FAKE_DRIVER = os.path.join(HERE, "fake_row_driver.py")
FAKE_ROWS = tuple(range(1, 13))


@dataclasses.dataclass
class WorldEleven:
    root: str
    base: object
    library: str
    environment: dict


def fake_world_eleven(root: str, hosts_code: int = 3) -> WorldEleven:
    base = two.fake_world_two(os.path.join(root, "world"), rows=",".join(map(str, FAKE_ROWS)), hosts_code=hosts_code)
    reuse.furnish_rows_world(base.repo, os.path.join(base.repo, "scripts/experiments/target/fake-frozen-tree"), FAKE_ROWS)
    wrapper = judge.write_text(os.path.join(root, "fake-driver-with-dep-info.py"), reuse.FAKE_DRIVER_WRAPPER.replace("FAKE_DRIVER_PATH", FAKE_DRIVER))
    environment = dict(base.environment, SEGMENT_TWO_R10_SELFTEST="1", SEGMENT_TWO_R10_SKIP_CONFIG_CHECK="1", SEGMENT_TWO_R10_YIELD_RECHECK_SECONDS="0.2",
                       SEGMENT_TWO_R10_RESEARCH_ROOT=two.REAL_REPOSITORY, SEGMENT_TWO_R11_SELFTEST="1", REUSE_SELFTEST="1", ROWS_ORCHESTRATOR_DRIVER=wrapper,
                       REUSE_SELFTEST_TOOLCHAIN="cargo fake\nrustc fake\n", REUSE_SELFTEST_BUILD_ENVIRONMENT="<构建环境：假> 0")
    return WorldEleven(root, base, os.path.join(root, "library"), environment)


def pass_config(world: WorldEleven, name: str, lookup: bool) -> str:
    """一遍的三份配置（照 rows_orchestrator_segment_two_revision_10.fake_world_ten 那一形；H23 各遍不同，修订 8 那一份与 T₂ 各遍相同）。交回修订 11 那一份。"""
    placeholders = layer10.placeholders
    tree = os.path.join(world.base.repo, "scripts/experiments/target/fake-frozen-tree")
    segment_one = placeholders.SegmentOne(tree="scripts/experiments/target/fake-frozen-tree", snapshot_digest=judge.row_driver.snapshot_digest(tree)[0],
                                          mutations_sha256=judge.sha256_of_file(os.path.join(tree, judge.MUTATIONS_TABLE)), run_date=layer10.FAKE_RUN_DATE,
                                          suffix="-r8", dumps="scripts/experiments/target/fake-segment-one-dumps", logs=os.path.join(world.root, "segment-one-logs"))
    judge.write_text(os.path.join(world.base.repo, segment_one.timing(placeholders.STEP_1_STEM)),
                     "E7TIMING name=stream_time stream=a elapsed_ns=50000000000\nE7TIMING name=process_wall seconds=75.274 exit=0\n")
    judge.write_text(segment_one.sequence_log, "\n".join(layer10.segment_one_lines(segment_one, world.base.repo, "finished")) + "\n")
    revision_8 = {"tree": segment_one.tree, "H3": {"snapshot_digest": segment_one.snapshot_digest}, "H8": {"mutations_table_sha256": segment_one.mutations_sha256},
                  "H16": {"run_date": layer10.FAKE_RUN_DATE, "run_suffix": "-r8", "dumps_root": segment_one.dumps, "logs_directory": segment_one.logs},
                  "H18": placeholders.revision_8.template()["H18"]}
    config = placeholders.template()
    config["revision_8_config"] = judge.write_text(os.path.join(world.root, f"revision-8-{name}.json"), json.dumps(revision_8, ensure_ascii=False))
    config["H23"] = {"run_date": f"2026-10-{name}", "run_suffix": "-r8", "dumps_root": f"scripts/experiments/target/e171-r1-dumps/fake-segment-two-{name}",
                     "logs_directory": os.path.join(world.root, f"segment-two-logs-{name}"), "work_root": os.path.join(world.root, f"segment-two-work-{name}")}
    revision_10 = judge.write_text(os.path.join(world.root, f"revision-10-{name}.json"), json.dumps(config, ensure_ascii=False))
    h31 = {path: reuse.sha256_file(path) for path in (os.path.abspath(__file__), os.path.join(HERE, "reuse_library.py"))}
    return judge.write_text(os.path.join(world.root, f"revision-11-{name}.json"),
                            json.dumps({"revision_10_config": revision_10, "H30": {"library_root": world.library, "lookup": lookup}, "H31": {"files": h31}}))


def run_pass(world: WorldEleven, name: str, lookup: bool, extra: dict[str, str] | None = None) -> tuple[int, str]:
    command = [sys.executable, "-B", os.path.abspath(__file__), "run", "--config", pass_config(world, name, lookup), "--repo", world.base.repo]
    completed = subprocess.run(command, cwd=world.base.repo, env=dict(world.environment, **(extra or {})), capture_output=True, text=True, timeout=300)
    return completed.returncode, completed.stdout + completed.stderr


def product_of(world: WorldEleven, name: str, stem: str) -> str:
    return os.path.join(world.base.repo, f"{layer10.placeholders.segment_two_prefix(f'2026-10-{name}')}-{stem}-r8.out")


def events_of(world: WorldEleven, name: str) -> list[str]:
    return reuse.event_lines(os.path.join(world.base.repo, f"scripts/experiments/target/e171-r1-dumps/fake-segment-two-{name}/reuse-markers", MARKERS_NAME))


def ran_rows(world: WorldEleven) -> list[str]:
    path = os.path.join(world.base.root, "driver.log")
    return [line.strip() for line in open(path, encoding="utf-8") if line.startswith("ran ")] if os.path.exists(path) else []


def merged_rows_without_timing(path: str) -> list[str]:
    return sorted(one.row_line_without_timing(line.rstrip("\n")) for line in open(path, encoding="utf-8") if line.startswith("row ")) if os.path.exists(path) else []


def cell_reuse_end_to_end(scratch: str) -> tuple[bool, str]:
    """只用本机：第一遍查库关（12 行全部现跑、入库）；第二遍查库开：跨机对照那一行（第 1 行，RC2）现跑，取来的 11 行里 RC3 挑 3 行现跑、与条目比都相同，
    其余 8 行取来不派；合成产物去耗时列与第一遍逐行相同；账本 reused=8、sample_size=3、check_mismatches=0。"""
    world = fake_world_eleven(scratch)
    first_code, _ = run_pass(world, "11", False)
    first_ran = len(ran_rows(world))
    second_code, output = run_pass(world, "12", True)
    second_ran = sorted(int(one.key_values(line)["row"]) for line in ran_rows(world)[first_ran:])
    events = events_of(world, "12")
    sample_line = [line for line in events if " name=reuse_sample " in f" {line} "]
    sampled = sorted(int(name.rsplit("-", 1)[1]) for name in sample_line[0].split(" rows=", 1)[1].split()[0].split(",")) if sample_line else []
    sample_checks = [line for line in events if " name=reuse_check " in f" {line} " and " kind=sample " in f" {line} "]
    ledger_path = product_of(world, "12", LEDGER_STEM)
    ledger = open(ledger_path, encoding="utf-8").read().strip().splitlines()[-1] if os.path.exists(ledger_path) else "账本不在"
    same = merged_rows_without_timing(product_of(world, "11", "rows-driver")) == merged_rows_without_timing(product_of(world, "12", "rows-driver")) != []
    good = (first_code == 0 and first_ran == 12 and second_code == 0 and len(sampled) == 3 and second_ran == sorted([1] + sampled)
            and len(sample_checks) == 3 and all(" identical=true " in f" {line} " for line in sample_checks) and same
            and " reused=8 " in ledger and " sample_size=3 " in ledger and " check_mismatches=0 " in ledger and " units_cover_registered=true " in ledger)
    return good, (f"第一遍退 {first_code}、真跑 {first_ran} 行；第二遍退 {second_code}、真跑第 {second_ran} 行（RC3 挑的 {sampled}）；抽样比 {len(sample_checks)} 次；"
                  f"产物去耗时列相同 {same}；账本 {ledger[22:140]}")


def cell_reuse_two_hosts(scratch: str) -> tuple[bool, str]:
    """两台：别的台取回的行入库之前核三样（假驱动的 mutation_line_sha256 不是变异行的 sha256，所以别的台的行都不入、各记一行 peer-check），
    本机的行照 RC4 入库；跨机对照那一份登记、永远现跑；账本 units=13。"""
    world = fake_world_eleven(scratch, hosts_code=0)
    code, output = run_pass(world, "21", False)
    events = events_of(world, "21")
    peer_refused = [line for line in events if "reason=peer-check-" in line]
    inserted = [line for line in events if " name=reuse_insert " in f" {line} " and " state=inserted " in f" {line} "]
    closure = [line for line in events if " name=reuse_closure_check " in f" {line} " and " matches=true " in f" {line} "]
    ledger_path = product_of(world, "21", LEDGER_STEM)
    ledger = open(ledger_path, encoding="utf-8").read().strip().splitlines()[-1] if os.path.exists(ledger_path) else "账本不在"
    good = (code == 0 and peer_refused and all("mutation_line_sha256" in line for line in peer_refused) and len(inserted) == len(closure) > 0
            and len(inserted) + len(peer_refused) == 13 and " units=13 " in ledger and " fresh_always=2 " in ledger)
    return good, f"退 {code}；别的台的行没入（peer-check）{len(peer_refused)} 行；本机入库 {len(inserted)} 行、RC4 相符 {len(closure)} 次；账本 {ledger[22:120]}"


def cell_peer_checks(scratch: str) -> tuple[bool, str]:
    raw = "m-row-two\tcrates/core/src/other.rs\talpha\tbeta\t-p core\tt_two"
    digest = reuse.sha256_bytes(raw.encode("utf-8"))
    decision = reuse.Decision("seg2-row-5", "driver-row", "k", f"[mutation]\nrow 5\nmutation {raw}\n[limit]\nseconds 1800\n", reuse.STATE_FRESH, "no-entry")
    STATE["frozen_snapshot_digest"] = "d" * 64
    outcomes = {}
    for case, row_number, line_digest, limit, snapshot in (("good", "5", digest, "1800", "d" * 64), ("row_number", "6", digest, "1800", "d" * 64),
                                                           ("mutation_line_sha256", "5", "0" * 64, "1800", "d" * 64),
                                                           ("row_time_limit", "5", digest, "5400", "d" * 64), ("snapshot_digest", "5", digest, "1800", "e" * 64)):
        row_directory = os.path.join(scratch, case, "row-5")
        judge.write_text(os.path.join(row_directory, "row-result.tsv"), f"row_number\t{row_number}\nmutation_line_sha256\t{line_digest}\n")
        product = judge.write_text(os.path.join(scratch, case, "product.out"), f"name=row_time_limit seconds={limit} baseline_seconds=1\nsnapshot_digest={snapshot} files=3\n")
        outcomes[case] = peer_check_problems(types.SimpleNamespace(shards=scratch), {"product": product}, 5, row_directory, decision)
    return outcomes["good"] == [] and all(outcomes[case] == [case] for case in outcomes if case != "good"), f"各样：{outcomes}"


def cell_s18_stops(scratch: str) -> tuple[bool, str]:
    world = fake_world_eleven(scratch)
    first_code, _ = run_pass(world, "31", False)
    second_code, output = run_pass(world, "32", False, {"FAKE_DRIVER_SALT_HOST": "1", "FAKE_DRIVER_SALT_PART": "dump"})
    frozen = reuse.Library(world.library).frozen_reason()
    return first_code == 0 and second_code == 1 and bool(frozen) and "S18 触发" in output, f"第一遍退 {first_code}；第二遍（导出掺字）退 {second_code}；库冻住 {bool(frozen)}"


SELFTEST_CELLS = {"reuse-e2e": cell_reuse_end_to_end, "reuse-two-hosts": cell_reuse_two_hosts, "peer-checks": cell_peer_checks, "s18-stops": cell_s18_stops}
SELFTEST_SWITCHES = {"sample-skipped": "reuse-e2e", "peer-checks-skipped": "peer-checks", "s18-ignored": "s18-stops"}


def selftest(argument_list: list[str]) -> int:
    os.environ[SELFTEST_VARIABLE] = "1"
    return harness.run_selftest(os.path.abspath(__file__), argument_list, SELFTEST_CELLS, SELFTEST_SWITCHES, BREAK_VARIABLE, SELFTEST_VARIABLE)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
