#!/usr/bin/env python3
"""E171 第 1 次重跑第一段·修订 8 那一趟：第 11–13 步的多机编排（登记 research/prompts/e171-r1-prereg.md 修订 8 第 8.4 节步表末行、第 8.9 节）。
由 segment_one_units_revision_8.py 在第 0、1、2、2b、7 步（与研究一侧变异表，排了的话）都绿之后起，在仓根跑。

为什么另写一层、不改第一段那一份编排器：rows_orchestrator.py 被 cross_host_o3_check.py、rows_orchestrator_segment_two.py 按 sha256 7bfbdb84… 钉住。
这一层按路径载入那一份（开跑先核 sha256，不是就退 3），行集、驱动参数、合并规则 ①–⑦、跨机一致对照（S17）的三样比法、第 13 步的规矩一字不改，调它原来的函数；
改的只有第 8.4 节步表末行点名的几处，与「-resume2 跨机一致对照 S17 停机的处置」第 5 条要带上的那一处：
  - 树、快照摘要、T 的来源（修订 8 树第 7 步的 -timing 产物，--baseline-timing 给）、产物前缀与后缀、导出根，都从占位配置读（H3、H16）；RESUME_TAG 不用（空）：
    行输出根 <导出根>/segment1-rows，产物 <前缀>-rows-…-r8.out（S17 的触发观测 …-rows-cross-host-check-r8.out）；
  - 别的台三个落点改名（H18：research/e171-s1-final-peer-kit、research/e171-s1-final-frozen-snapshot、scripts/experiments/target/e171-s1-final-peer-out），
    多机标签前缀改成 e171s1r8，不与修订 6 那一趟同名；
  - 每行前后（编排器每个驱动起之前、收尾之后核 Z5 的那两处）另核树里 crates/mutations.tsv 等于 H8，不等记作废 Z5、停；
  - 跨机对照 o3 那一格换成 cross_host_o3_check.py（按路径载入、核 sha256）的比法：原去法之后再去 name=state 的 worker=、name=timing 的 workers=、
    worker_processes= 并按整行排序比多重集；前三格仍调编排器原函数；
  - 停派文件（--stop-file）：单元编排在第 3–10 步某个单元红时写它，这一层见到就不再派新行，在跑的跑完再停（第 8.4 节「次序」第 5 条）；
  - 加槽：逐行与第 3–10 步还在跑的单元共用本机，起跑时现查的余量可能偏少；这一层每 REGROW_SECONDS 秒重查一次各台，那一台最近 REGROW_SECONDS 秒里
    没起过行、此刻放得下的就照编排器的分配法给它加槽（别的台先过预检）。第 13 步的单线道不加槽。排队里只剩放不下的行时先重查一次再收尾。
用法：
  rows_orchestrator_revision_8.py run  --config C --repo R --baseline-timing B --stop-file S
  rows_orchestrator_revision_8.py plan --config C --repo R --baseline-timing B --stop-file S   （只现查、算分配、推各台挂钟，打出来；一行都不起）
  rows_orchestrator_revision_8.py --selftest [--cells 格,…] [--rounds N]
退出码：0 跑完；1 停机或某一行没跑成；2 用法错；3 编排器或跨机对照入口不是钉的那一份；78 运行条件不满足；143 收到 TERM / INT / HUP。

弄坏开关 ROWS_R8_BREAK=<项>（只在 --selftest 起的子进程里认，ROWS_R8_SELFTEST=1）。项名、弄坏哪一处、打开后红在哪一格：
  peer-names-revision-6       别的台的落点不改名                             overrides-applied
  mutations-table-skipped     每行前后不核树里 crates/mutations.tsv           mutations-table-checked
  cross-host-original-o3      跨机对照 o3 那一格用编排器原来的比法           cross-host-o3-scheduling-keys
  stop-file-ignored           见到停派文件照样派新行                         e2e-stop-file
  regrow-skipped              不加槽                                         e2e-regrow
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import re
import subprocess
import sys
import time
import types

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import frozen_tree_lines_revision_8 as judge  # noqa: E402
import revision_8_placeholders as placeholders  # noqa: E402
import revision_8_selftest_harness as harness  # noqa: E402

BREAK_VARIABLE = "ROWS_R8_BREAK"
SELFTEST_VARIABLE = "ROWS_R8_SELFTEST"
SEGMENT_ONE_ORCHESTRATOR_PATH = os.path.join(HERE, "rows_orchestrator.py")
SEGMENT_ONE_ORCHESTRATOR_SHA256 = "7bfbdb84559476033853794499836f752fdf8a3e803e17cd9539ae49ee3b7e0e"
CROSS_HOST_CHECK_PATH = os.path.join(HERE, "cross_host_o3_check.py")
CROSS_HOST_CHECK_SHA256 = "afc9a9b081b90e125c4050d01215dbbca2cf7f088604faa66243b23590c2bcc6"
SEGMENT_ONE_SELFTEST_LAST_LINE = "自证 34 项，红 0"
CROSS_HOST_SELFTEST_LAST_LINE = "自证 38 项，红 0"
PEER_LABEL_PREFIX = "e171s1r8"
EXIT_WRONG_PINNED_FILE = 3
REGROW_SECONDS = 600.0


class WrongPinnedFile(Exception):
    """编排器或跨机对照入口的 sha256 不是钉的那一份。"""


def load_pinned(path: str, expected: str, module_name: str) -> types.ModuleType:
    """按路径载入（不改它、不写 .pyc）；sha256 不对就拒。"""
    with open(path, "rb") as handle:
        actual = hashlib.sha256(handle.read()).hexdigest()
    if actual != expected:
        raise WrongPinnedFile(f"{path} 的 sha256 是 {actual}，不是 {expected}")
    specification = importlib.util.spec_from_file_location(module_name, path)
    module = importlib.util.module_from_spec(specification)
    sys.dont_write_bytecode = True
    sys.modules[specification.name] = module  # dataclasses 在类定义时按 __module__ 去 sys.modules 里找这个模块
    specification.loader.exec_module(module)
    return module


try:
    one = load_pinned(SEGMENT_ONE_ORCHESTRATOR_PATH, SEGMENT_ONE_ORCHESTRATOR_SHA256, "rows_orchestrator_7bfbdb84")
    cross_host = load_pinned(CROSS_HOST_CHECK_PATH, CROSS_HOST_CHECK_SHA256, "cross_host_o3_check_pinned")
except WrongPinnedFile as wrong_file:
    print(f"✗ {wrong_file}（这一层调的是钉住的那一份的函数；改过就先核这一层，再改钉的 sha256）", file=sys.stderr)
    sys.exit(EXIT_WRONG_PINNED_FILE)


def broken(item: str) -> bool:
    return harness.broken(BREAK_VARIABLE, item)


def regrow_seconds() -> float:
    if os.environ.get(SELFTEST_VARIABLE) == "1" and os.environ.get("ROWS_R8_REGROW_SECONDS"):
        return float(os.environ["ROWS_R8_REGROW_SECONDS"])
    return REGROW_SECONDS


# ── 换进编排器的几处（configure 一次装好；编排器里的函数按模块全局名取，换了就用这里的） ──
STATE: dict = {"mutations_sha256": None, "stop_file": None, "regrow": None}
ORIGINAL = {name: getattr(one, name) for name in ("snapshot_check", "cross_host_parts", "cross_host_product", "pick_job", "run_queue",
                                                   "phase_rows_and_checks", "peer_label")}


def snapshot_check_with_mutations_table(context) -> None:
    """作废 Z5（编排器原核快照摘要）＋修订 8 第 8.6 节：树里 crates/mutations.tsv 等于 H8。"""
    ORIGINAL["snapshot_check"](context)
    if broken("mutations-table-skipped"):
        return
    actual = judge.sha256_or_missing(os.path.join(context.repo, context.tree, judge.MUTATIONS_TABLE))
    if actual != STATE["mutations_sha256"]:
        raise one.Stopped(f"树里 crates/mutations.tsv 变了（作废 Z5，H8）：{actual}")


def cross_host_parts_without_scheduling_keys(line_a: str, directory_a: str, line_b: str, directory_b: str):
    if broken("cross-host-original-o3"):
        return ORIGINAL["cross_host_parts"](line_a, directory_a, line_b, directory_b)
    return cross_host.cross_host_parts(one, line_a, directory_a, line_b, directory_b)


def cross_host_product_of_revision_8(parts, host_a, host_b, reason):
    if parts is None or broken("cross-host-original-o3"):
        return ORIGINAL["cross_host_product"](parts, host_a, host_b, reason)
    inputs = (f"E7RESULT name=cross_host_inputs entry=rows_orchestrator_revision_8.py orchestrator_sha256={SEGMENT_ONE_ORCHESTRATOR_SHA256} "
              f"cross_host_o3_check_sha256={CROSS_HOST_CHECK_SHA256}")
    return cross_host.cross_host_product(one, parts, host_a, host_b, inputs)


def pick_job_unless_stopped(queue, host, row_zero_host):
    stop_file = STATE["stop_file"]
    if stop_file and os.path.exists(stop_file) and not broken("stop-file-ignored"):
        return None
    return ORIGINAL["pick_job"](queue, host, row_zero_host)


def peer_label_of_revision_8(context, label: str) -> str:
    return re.sub(r"[^A-Za-z0-9._-]", "-", f"{PEER_LABEL_PREFIX}{context.tag}-{os.path.basename(context.attempt_dir)}-{label}")


def phase_rows_and_checks_with_regrow(context, hosts, allocations, setup, expected_hashes) -> None:
    """第 11、12 步那一个队列加槽要的台与预检（第 13 步单线道不加槽：它调 run_queue 时这里已清掉）。"""
    STATE["regrow"] = {"hosts": hosts, "setup": setup}
    try:
        ORIGINAL["phase_rows_and_checks"](context, hosts, allocations, setup, expected_hashes)
    finally:
        STATE["regrow"] = None


def configure(config: dict, stop_file: str) -> None:
    h18 = judge.section(config, "H18")
    if not broken("peer-names-revision-6"):
        one.PEER_KIT_RELATIVE, one.PEER_SNAPSHOT_RELATIVE, one.PEER_OUT_RELATIVE = h18["peer_kit"], h18["peer_snapshot"], h18["peer_out"]
        one.peer_label = peer_label_of_revision_8
    STATE["mutations_sha256"] = judge.section(config, "H8").get("mutations_table_sha256")
    STATE["stop_file"] = stop_file
    one.snapshot_check = snapshot_check_with_mutations_table
    one.cross_host_parts = cross_host_parts_without_scheduling_keys
    one.cross_host_product = cross_host_product_of_revision_8
    one.pick_job = pick_job_unless_stopped
    one.run_queue = run_queue_with_regrow
    one.phase_rows_and_checks = phase_rows_and_checks_with_regrow


def run_queue_with_regrow(context, allocations, jobs, row_zero_host, expected_hashes, kit, configurations):
    """编排器 run_queue（rows_orchestrator.py 第 988–1042 行）照抄，改三处：每槽的线程按槽记（加出来的槽线程可能不同）；
    加槽（文件头「加槽」：只在第 11、12 步那个队列里，STATE["regrow"] 有台与预检时）；起分片时别的台用 setup 那一份 kit（加槽时才建的也算）。
    派不派新行仍由 pick_job 定（停派文件在那里拦）。交回 (跑成的分片记录, 放不下的, 没跑成的说明)。"""
    regrow = None if broken("regrow-skipped") else STATE["regrow"]
    free = [(allocation.number, index) for allocation in allocations for index in range(allocation.slots)]
    threads_of_slot = {(allocation.number, index): allocation.threads_per_slot for allocation in allocations for index in range(allocation.slots)}
    last_start: dict[int, float] = {}
    last_regrow = time.monotonic()
    queue = one.queue_order(jobs)
    running: list = []
    succeeded: list[dict] = []
    failures: list[str] = []

    def try_regrow() -> int:
        hosts, setup = regrow["hosts"], regrow["setup"]
        try:
            resources = one.query_resources(context, hosts)
        except one.RunCondition as error:
            one.say(context, f"  加槽：这一次没读到资源（{error}），不加")
            return 0
        added = 0
        for allocation in one.allocate(hosts, resources):
            if allocation.slots <= 0 or time.monotonic() - last_start.get(allocation.number, -1e18) < regrow_seconds():
                continue
            try:
                setup.ready(allocation.number, allocation.slots, allocation.threads_per_slot)
            except one.RunCondition as error:
                one.say(context, f"  加槽：{one.host_name(allocation.number)}这一次没过预检（{error}），不加")
                continue
            base = max([index for host, index in threads_of_slot if host == allocation.number], default=-1) + 1
            for index in range(base, base + allocation.slots):
                free.append((allocation.number, index))
                threads_of_slot[(allocation.number, index)] = allocation.threads_per_slot
            added += allocation.slots
            one.say(context, f"  加槽：{one.host_name(allocation.number)}加 {allocation.slots} 槽，每槽 {allocation.threads_per_slot} 线程（现查的余量）")
        return added

    while not one.STOP.is_set():
        for slot in sorted(free) if not failures else []:
            chosen = one.pick_job(queue, slot[0], row_zero_host)
            if chosen is None:
                continue
            job = queue.pop(chosen)
            step = one.STEPS[job.step]
            label = f"{job.step}-row-{'-'.join(str(row) for row in job.rows)}-h{slot[0]}"
            try:
                one.snapshot_check(context)  # 作废 Z5（另核 H8）：每个驱动起之前核一次
                if slot[0] == 1:
                    fresh = 0 in job.rows or job.step in ("r11", "r12")
                    work_root = os.path.join(context.work_root, f"{os.path.basename(context.attempt_dir)}-{label}" if fresh else f"host1-slot{slot[1]}")
                    process, record = one.launch_local(context, expected_hashes, label, step, job.rows, 1, threads_of_slot[slot], work_root, "local")
                else:
                    peer_kit = regrow["setup"].kit if regrow else kit
                    peer_configurations = regrow["setup"].configurations if regrow else configurations
                    assert peer_kit is not None, "有别的台的槽就建过发给它的那一份"
                    process, record = one.launch_peer(context, peer_kit, label, step, job.rows, threads_of_slot[slot], slot[0], peer_configurations.get(slot[0]))
            except one.Stopped as error:
                failures.append(str(error))
                break
            if job.step == "s12" and 0 in job.rows:
                row_zero_host = slot[0]
            free.remove(slot)
            last_start[slot[0]] = time.monotonic()
            running.append(one.Running(job, slot, process, record))
            one.say(context, f"  ▶ 第 {step.number} 步 {job.step} 第 {one.rows_text(job.rows)} 行 → {one.host_name(slot[0])} 第 {slot[1]} 槽（{threads_of_slot[slot]} 线程）")
        for item in list(running):
            code = item.process.poll()
            if code is None:
                continue
            running.remove(item)
            free.append(item.slot)
            if one.finish_shard(context, item.record, code):
                succeeded.append(item.record)
            else:
                failures.append(f"{item.job.step} 第 {one.rows_text(item.job.rows)} 行（{one.host_name(item.slot[0])}，退 {code}，见 {item.record['label']}/）")
            try:
                one.snapshot_check(context)  # 作废 Z5（另核 H8）：每个驱动收尾再核一次
            except one.Stopped as error:
                failures.append(str(error))
        stuck = bool(queue) and all(one.pick_job(queue, slot[0], row_zero_host) is None for slot in free)
        stop_file = STATE["stop_file"]
        stopped = bool(stop_file) and os.path.exists(stop_file) and not broken("stop-file-ignored")
        if regrow and queue and not failures and not stopped and (time.monotonic() - last_regrow >= regrow_seconds() or (stuck and not running)):
            last_regrow = time.monotonic()
            if try_regrow():
                stuck = all(one.pick_job(queue, slot[0], row_zero_host) is None for slot in free)
        if not running and (failures or not queue or stuck):
            break
        time.sleep(context.poll_seconds)
    if one.STOP.is_set():
        one.stop_running(context, running)
        raise one.Interrupted()
    return succeeded, queue, failures


def context_of(config: dict, repo: str, baseline_timing: str):
    h16 = judge.section(config, "H16")
    logs = str(h16.get("logs_directory"))
    arguments = argparse.Namespace(repo=repo, prefix=placeholders.product_prefix(str(h16.get("run_date"))), suffix=str(h16.get("run_suffix")), resume_tag="",
                                   dumps=str(h16.get("dumps_root")), tree=judge.tree_of(config),
                                   snapshot_digest=str(judge.section(config, "H3").get("snapshot_digest")), baseline_timing=baseline_timing,
                                   check=os.path.join(HERE, "check_product.py"), logs=logs, work_root=os.path.join(logs, "rows-work"), research_root=None)
    return one.context_from_arguments(arguments)


def main(argument_list: list[str]) -> int:
    for variable, selftest_variable in ((BREAK_VARIABLE, SELFTEST_VARIABLE), ("ROWS_ORCHESTRATOR_BREAK", "ROWS_ORCHESTRATOR_SELFTEST")):
        refusal = harness.refuse_break_outside_selftest(variable, selftest_variable)
        if refusal:
            print(refusal, file=sys.stderr)
            return 2
    if argument_list[:1] == ["--selftest"]:
        return selftest(argument_list[1:])
    parser = argparse.ArgumentParser(description="E171 修订 8 那一趟：第 11–13 步的多机编排（第一段编排器外面一层）")
    parser.add_argument("mode", choices=("run", "plan"))
    for name in ("--config", "--baseline-timing", "--stop-file"):
        parser.add_argument(name, required=True)
    parser.add_argument("--repo", default=os.getcwd())
    arguments = parser.parse_args(argument_list)
    try:
        config = judge.load_config(arguments.config)
    except judge.RunConditionError as error:
        print(f"✗ {error}", file=sys.stderr)
        return one.EXIT_RUN_CONDITION
    configure(config, arguments.stop_file)
    context = context_of(config, os.path.abspath(arguments.repo), arguments.baseline_timing)
    try:
        return one.run(context, arguments.mode == "plan")
    except one.RunCondition as error:
        one.say(context, f"✗ 运行条件不满足（退 78）：{error}")
        return one.EXIT_RUN_CONDITION
    except one.Stopped as error:
        one.say(context, f"✗ 停：{error}")
        return 1
    except one.Interrupted:
        one.say(context, "收到停止信号：在跑的各行已停、别的台已做完的行已取回收进来；同一条命令再起就是续跑")
        return one.EXIT_STOPPED_BY_SIGNAL


# ── 自证：借第一段编排器的假世界（fake_world：假驱动、假 multi-host-run、假机器清单与资源），起的是这一层 ──
FAKE_RUN_DATE = "2026-10-09"


def fake_config(root: str, world, peer_names: tuple[str, str, str] | None = None) -> str:
    tree = os.path.join(world.repo, "scripts/experiments/target/fake-frozen-tree")
    judge.write_text(os.path.join(tree, judge.MUTATIONS_TABLE), "# 假的坏实现表\n")
    names = peer_names or ("research/e171-s1-final-peer-kit", "research/e171-s1-final-frozen-snapshot", "scripts/experiments/target/e171-s1-final-peer-out")
    config = {"tree": "scripts/experiments/target/fake-frozen-tree", "H3": {"snapshot_digest": judge.row_driver.snapshot_digest(tree)[0]},
              "H8": {"mutations_table_sha256": judge.sha256_of_file(os.path.join(tree, judge.MUTATIONS_TABLE))},
              "H16": {"run_date": FAKE_RUN_DATE, "run_suffix": "-r8", "dumps_root": world.dumps, "logs_directory": os.path.join(root, "logs")},
              "H18": dict(zip(("peer_kit", "peer_snapshot", "peer_out"), names))}
    prefix = os.path.join(world.repo, placeholders.product_prefix(FAKE_RUN_DATE))
    for stem, seconds in (("baseline-run1", "100.500"), ("positive-control-o1-crash-judge", "80.000"), ("positive-control-reference-filesystem", "70.000")):
        judge.write_text(f"{prefix}-{stem}-r8-timing.out", f"E7TIMING name=stream_time stream=a elapsed_ns=1000000000\nE7TIMING name=process_wall seconds={seconds} exit=0\n")
    return judge.write_text(os.path.join(root, "config.json"), json.dumps(config))


def wrapper_command(world, config_path: str, stop_file: str, mode: str = "run") -> list[str]:
    return [sys.executable, "-B", os.path.abspath(__file__), mode, "--config", config_path, "--repo", world.repo,
            "--baseline-timing", placeholders.product_prefix(FAKE_RUN_DATE) + "-baseline-run1-r8-timing.out", "--stop-file", stop_file]


def product_path(world, stem: str) -> str:
    return os.path.join(world.repo, f"{placeholders.product_prefix(FAKE_RUN_DATE)}-{stem}-r8.out")


def cell_overrides_applied(scratch: str) -> tuple[bool, str]:
    config = judge.load_config(os.path.join(HERE, "revision_8_placeholders.json"))
    configure(config, os.path.join(scratch, "stop"))
    names = (one.PEER_KIT_RELATIVE, one.PEER_SNAPSHOT_RELATIVE, one.PEER_OUT_RELATIVE)
    expected = tuple(judge.section(config, "H18")[key] for key in ("peer_kit", "peer_snapshot", "peer_out"))
    label = one.peer_label(types.SimpleNamespace(tag="", attempt_dir="/x/attempt-1"), "s12-row-0-h2")
    good = names == expected and label.startswith(PEER_LABEL_PREFIX) and one.snapshot_check is snapshot_check_with_mutations_table \
        and one.run_queue is run_queue_with_regrow and one.cross_host_parts is cross_host_parts_without_scheduling_keys
    return good, f"三个落点 {names}；标签 {label}"


def cell_mutations_table_checked(scratch: str) -> tuple[bool, str]:
    world = one.fake_world(os.path.join(scratch, "world"))
    config_path = fake_config(scratch, world)
    config = judge.load_config(config_path)
    configure(config, os.path.join(scratch, "stop"))
    context = context_of(config, world.repo, "unused")
    one.snapshot_check(context)
    judge.write_text(os.path.join(world.repo, "scripts/experiments/target/fake-frozen-tree", judge.MUTATIONS_TABLE), "# 拍完又改了\n")
    try:
        one.snapshot_check(context)
    except one.Stopped as error:
        return "H8" in str(error), f"树里 crates/mutations.tsv 改了：停（{error}）"
    return False, "树里 crates/mutations.tsv 改了却没停"


def cell_cross_host_o3_scheduling_keys(scratch: str) -> tuple[bool, str]:
    line_a = one.write_fake_row(os.path.join(scratch, "a"), workers=5)
    line_b = one.write_fake_row(os.path.join(scratch, "b"), workers=7)
    for directory, workers, order in (("a", 5, (1, 2)), ("b", 7, (2, 1))):
        with open(os.path.join(scratch, directory, "o3.out"), "a", encoding="utf-8") as handle:
            for state in order:
                handle.write(f"name=state stream=s state={state} batch=1 worker={4000 + workers * state} strict=in digest=d{state}\n")
            handle.write(f"name=timing batch=1 workers={workers} worker_processes={workers - 1} judged_states=2\n")
    configure(judge.load_config(os.path.join(HERE, "revision_8_placeholders.json")), os.path.join(scratch, "stop"))
    wrapped = {name: same for name, same, _ in one.cross_host_parts(line_a, os.path.join(scratch, "a"), line_b, os.path.join(scratch, "b"))}
    original = {name: same for name, same, _ in ORIGINAL["cross_host_parts"](line_a, os.path.join(scratch, "a"), line_b, os.path.join(scratch, "b"))}
    return wrapped.get("o3") is True and original.get("o3") is False and all(wrapped.values()), f"去调度键后 {wrapped}；原比法 o3={original.get('o3')}"


def run_world(world, config_path: str, stop_file: str, extra: dict[str, str] | None = None, timeout: float = 240) -> tuple[int, str]:
    environment = dict(world.environment, ROWS_R8_SELFTEST="1", **(extra or {}))
    completed = subprocess.run(wrapper_command(world, config_path, stop_file), cwd=world.repo, env=environment, capture_output=True, text=True, timeout=timeout)
    return completed.returncode, completed.stdout + completed.stderr


def ledger_text(world) -> str:
    path = os.path.join(world.repo, world.dumps, "segment1-rows-shards", "row-hosts.tsv")
    return open(path, encoding="utf-8").read() if os.path.exists(path) else ""


def cell_e2e_two_hosts(scratch: str) -> tuple[bool, str]:
    world = one.fake_world(os.path.join(scratch, "world"))
    config_path = fake_config(scratch, world)
    code, output = run_world(world, config_path, os.path.join(scratch, "stop"))
    verdict = one.verdict_of(product_path(world, one.CROSS_HOST_CHECK_STEM))
    fetched_paths = [root for root, directories, _ in os.walk(os.path.join(world.repo, world.dumps, "segment1-rows-shards"))
                     if root.endswith("scripts/experiments/target/e171-s1-final-peer-out")]
    good = (code == 0 and os.path.exists(product_path(world, "rows-driver")) and os.path.exists(product_path(world, "rows-probe-driver"))
            and "cross_host_identical=true" in verdict and "fetched" in ledger_text(world) and bool(fetched_paths)
            and "cross_host_o3_check_sha256=" in open(product_path(world, one.CROSS_HOST_CHECK_STEM), encoding="utf-8").read())
    return good, f"退 {code}；跨机 {verdict[:70]}；取回到新落点的分片 {len(fetched_paths)}；末行 {output.strip().splitlines()[-1:]}"


def cell_e2e_stop_file(scratch: str) -> tuple[bool, str]:
    world = one.fake_world(os.path.join(scratch, "world"))
    config_path = fake_config(scratch, world)
    stop_file = judge.write_text(os.path.join(scratch, "stop"), "单元 step-3-o2 红\n")
    code, output = run_world(world, config_path, stop_file)
    ran = sum(1 for line in open(os.path.join(world.root, "driver.log"), encoding="utf-8") if line.startswith("ran ")) \
        if os.path.exists(os.path.join(world.root, "driver.log")) else 0
    return code == 1 and ran == 0, f"有停派文件：退 {code}，真跑过的行 {ran}（要 0）"


def cell_e2e_regrow(scratch: str) -> tuple[bool, str]:
    peer_empty = [one.resources_entry(1, 5, 25, 25), one.resources_entry(2, 0, 25, 25)]
    world = one.fake_world(os.path.join(scratch, "world"), resources=peer_empty, extra_environment={"FAKE_DRIVER_ROW_SECONDS": "0.2"})
    config_path = fake_config(scratch, world)
    environment = dict(world.environment, ROWS_R8_SELFTEST="1", ROWS_R8_REGROW_SECONDS="0.3")
    process = subprocess.Popen(wrapper_command(world, config_path, os.path.join(scratch, "stop")), cwd=world.repo, env=environment,
                               stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    log = os.path.join(world.root, "driver.log")
    deadline = time.monotonic() + 60
    while time.monotonic() < deadline and not (os.path.exists(log) and "ran " in open(log, encoding="utf-8").read()):
        time.sleep(0.02)
    judge.write_text(os.path.join(world.root, "resources.json"), json.dumps({"hosts": [one.resources_entry(1, 0, 25, 25), one.resources_entry(2, 10, 25, 25)]}))
    output, _ = process.communicate(timeout=240)
    good = process.returncode == 0 and "加槽：第二台" in output and "\t2\t" in ledger_text(world)
    return good, f"起跑时第二台 0 槽、跑起来之后放得下：退 {process.returncode}；加槽那一行 {'有' if '加槽：第二台' in output else '没有'}；末行 {output.strip().splitlines()[-1:]}"


def pinned_selftest_cell(path: str, last_line: str) -> tuple[bool, str]:
    environment = {key: value for key, value in os.environ.items() if not key.startswith(("ROWS_R8_", "ROWS_ORCHESTRATOR_", "CROSS_HOST_O3_"))}
    completed = subprocess.run([sys.executable, "-B", path, "--selftest"], capture_output=True, text=True, env=environment, timeout=900)
    got = completed.stdout.strip().splitlines()[-1:] or [""]
    return completed.returncode == 0 and got[0] == last_line, f"{os.path.basename(path)} --selftest 末行 {got[0]}（要 {last_line}）"


SELFTEST_CELLS = {
    "overrides-applied": cell_overrides_applied, "mutations-table-checked": cell_mutations_table_checked,
    "cross-host-o3-scheduling-keys": cell_cross_host_o3_scheduling_keys, "e2e-two-hosts": cell_e2e_two_hosts, "e2e-stop-file": cell_e2e_stop_file,
    "e2e-regrow": cell_e2e_regrow,
    "segment-one-orchestrator-selftest": lambda scratch: pinned_selftest_cell(SEGMENT_ONE_ORCHESTRATOR_PATH, SEGMENT_ONE_SELFTEST_LAST_LINE),
    "cross-host-check-selftest": lambda scratch: pinned_selftest_cell(CROSS_HOST_CHECK_PATH, CROSS_HOST_SELFTEST_LAST_LINE),
}
SELFTEST_SWITCHES = {"peer-names-revision-6": "overrides-applied", "mutations-table-skipped": "mutations-table-checked",
                     "cross-host-original-o3": "cross-host-o3-scheduling-keys", "stop-file-ignored": "e2e-stop-file", "regrow-skipped": "e2e-regrow"}


def selftest(argument_list: list[str]) -> int:
    os.environ[SELFTEST_VARIABLE] = "1"
    return harness.run_selftest(os.path.abspath(__file__), argument_list, SELFTEST_CELLS, SELFTEST_SWITCHES, BREAK_VARIABLE, SELFTEST_VARIABLE)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
