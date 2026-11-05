#!/usr/bin/env python3
# task-server-shards: run
"""E171 第 1 次重跑第一段·修订 8 那一趟：第 11–13 步的多机编排（登记 research/prompts/e171-r1-prereg.md 修订 8 第 8.4 节步表末行、第 8.9 节）。
由 segment_one_units_revision_8.py 在第 0、1、2、2b、7 步（与研究一侧变异表，排了的话）都绿之后起，在仓根跑。

为什么另写一层、不改第一段那一份编排器：rows_orchestrator.py 被 cross_host_o3_check.py、rows_orchestrator_segment_two.py 按 sha256 钉住。
这一层按路径载入那一份（开跑先核 sha256，不是就退 3），行集、驱动参数、合并规则 ①–⑦、跨机一致对照（S17）的三样比法、第 13 步的规矩一字不改，调它原来的函数；
改的只有第 8.4 节步表末行点名的几处，与「-resume2 跨机一致对照 S17 停机的处置」第 5 条要带上的那一处：
  - 树、快照摘要、T 的来源（修订 8 树第 7 步的 -timing 产物，--baseline-timing 给）、产物前缀与后缀、导出根，都从占位配置读（H3、H16）；RESUME_TAG 不用（空）：
    行输出根 <导出根>/segment1-rows，产物 <前缀>-rows-…-r8.out（S17 的触发观测 …-rows-cross-host-check-r8.out）；
  - 多机标签前缀改成 e171s1r8，不与修订 6 那一趟同名；H18 的三个落点不再用：别的台照同一个绝对路径读写共享根（rows_orchestrator.py 文件头「机器」，
    不送、不取回），工具包、快照与输出都在这一趟自己的分片目录里，不与修订 6 那一趟撞名；
  - 每行前后（编排器每个驱动起之前、收尾之后核 Z5 的那两处）另核树里 crates/mutations.tsv 等于 H8，不等记作废 Z5、停；
  - 跨机对照 o3 那一格换成 cross_host_o3_check.py（按路径载入、核 sha256）的比法：原去法之后再去 name=state 的 worker=、name=timing 的 workers=、
    worker_processes= 并按整行排序比多重集；前三格仍调编排器原函数；
  - 停派文件（--stop-file）：单元编排在第 3–10 步某个单元红时写它，这一层见到就不再派新行，在跑的跑完再停（第 8.4 节「次序」第 5 条）；
  - 运行条件（第 2 次重跑登记 research/prompts/e171-r2-prereg.md 修订 4）：H16 缺一项就退 78、点名缺哪几项；冻结树不在或快照是空集也退 78，
    不拿空集的摘要往下判 Z5；
经 task-server：文件头第二行声明了 run 这一种分片，服务照计划、各片、合并三步起（rows_orchestrator.py 文件头「经 task-server」）；
  不经服务直接调（segment_one_units_revision_8.py 起的那一下）只用这一台，跨机对照报 not_run。
用法：
  rows_orchestrator_revision_8.py run  --config C --repo R --baseline-timing B --stop-file S
  rows_orchestrator_revision_8.py plan --config C --repo R --baseline-timing B --stop-file S   （只现查、算分配、推各台挂钟，打出来；一行都不起）
  rows_orchestrator_revision_8.py --selftest [--cells 格,…] [--rounds N]
退出码：0 跑完；1 停机或某一行没跑成；2 用法错；3 编排器或跨机对照入口不是钉的那一份；78 运行条件不满足；143 收到 TERM / INT / HUP。

弄坏开关 ROWS_R8_BREAK=<项>（只在 --selftest 起的子进程里认，ROWS_R8_SELFTEST=1；平常跑设了退 2）。逐项写开关名、弄坏哪一处，
下一行起写打开后红的格（自证打「✗ 自证 <格名>：…」；开关轮只核每项第一格）：
  ROWS_R8_BREAK=peer-names-revision-6  别的台的标签前缀不改名（照修订 6 那一趟的）
      「✗ 自证 overrides-applied：」「✗ 自证 e2e-two-hosts：」
  ROWS_R8_BREAK=mutations-table-skipped  每行前后不核树里 crates/mutations.tsv
      「✗ 自证 mutations-table-checked：」
  ROWS_R8_BREAK=cross-host-original-o3  跨机对照 o3 那一格用编排器原来的比法
      「✗ 自证 cross-host-o3-scheduling-keys：」「✗ 自证 e2e-two-hosts：」
  ROWS_R8_BREAK=stop-file-ignored  见到停派文件照样派新行
      「✗ 自证 e2e-stop-file：」
  ROWS_R8_BREAK=sweep-skipped  每一格之后与末尾不收进程（自证外壳 revision_8_selftest_harness.py 的开关）
      「✗ 自证 no-leftover-processes：」
  ROWS_R8_BREAK=temporary-kept  末尾不删临时目录（自证外壳 revision_8_selftest_harness.py 的开关）
      「✗ 自证 temporary-directory-removed：」
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
import types

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import frozen_tree_lines_revision_8 as judge  # noqa: E402
import revision_8_placeholders as placeholders  # noqa: E402
import revision_8_selftest_harness as harness  # noqa: E402

BREAK_VARIABLE = "ROWS_R8_BREAK"
SELFTEST_VARIABLE = "ROWS_R8_SELFTEST"
SEGMENT_ONE_ORCHESTRATOR_PATH = os.path.join(HERE, "rows_orchestrator.py")
# 两个钉值都是被钉文件此刻的内容：跑那一趟的第一段编排器 1dcfb8d0… 那一版与此刻差的不只是登记过的搬迁（每台线程改取登记值等），倒换不回去
SEGMENT_ONE_ORCHESTRATOR_SHA256 = "d2ebaaaf914db01f384004676d9e127593c11d1796a4694b50924db38b2255c1"
CROSS_HOST_CHECK_PATH = os.path.join(HERE, "cross_host_o3_check.py")
CROSS_HOST_CHECK_SHA256 = "7247276e8046551e18bbfe2a3402babfb3e4e8554f467bac56796c2e6975fc42"
SEGMENT_ONE_SELFTEST_LAST_LINE = "自证 35 项，红 0"
CROSS_HOST_SELFTEST_LAST_LINE = "自证 38 项，红 0"
PEER_LABEL_PREFIX = "e171s1r8"
EXIT_WRONG_PINNED_FILE = 3


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


def refuse_break_switches_outside_selftest() -> bool:
    """平常跑（不是自证起的子进程）设了这一层或第一段编排器的弄坏开关：打拒因，交回 True（调用方退 2）。"""
    for variable, selftest_variable in ((BREAK_VARIABLE, SELFTEST_VARIABLE), ("ROWS_ORCHESTRATOR_BREAK", "ROWS_ORCHESTRATOR_SELFTEST")):
        refusal = harness.refuse_break_outside_selftest(variable, selftest_variable)
        if refusal:
            print(refusal, file=sys.stderr)
            print('     → 怎么办：平常跑时 unset 那个变量再起；它只给 --selftest 起的子进程用', file=sys.stderr)
            return True
    return False


# 守卫在下面按 sha256 载入被钉的两份之前：平常跑设了弄坏开关先退 2，不让钉值核对的退 3 把它盖住、也不先载被钉的那两份
if __name__ == "__main__" and refuse_break_switches_outside_selftest():
    sys.exit(2)
try:
    one = load_pinned(SEGMENT_ONE_ORCHESTRATOR_PATH, SEGMENT_ONE_ORCHESTRATOR_SHA256, "rows_orchestrator_1dcfb8d0")
    cross_host = load_pinned(CROSS_HOST_CHECK_PATH, CROSS_HOST_CHECK_SHA256, "cross_host_o3_check_pinned")
except WrongPinnedFile as wrong_file:
    print(f"✗ {wrong_file}（这一层调的是钉住的那一份的函数；改过就先核这一层，再改钉的 sha256）", file=sys.stderr)
    print("  → 怎么办：对着点名的那份文件核这一层调的函数还在不在、行为变没变，核过再把钉的 sha256 换成它此刻的值", file=sys.stderr)
    sys.exit(EXIT_WRONG_PINNED_FILE)


def broken(item: str) -> bool:
    return harness.broken(BREAK_VARIABLE, item)


# ── 换进编排器的几处（configure 一次装好；编排器里的函数按模块全局名取，换了就用这里的） ──
STATE: dict = {"mutations_sha256": None, "stop_file": None}
ORIGINAL = {name: getattr(one, name) for name in ("snapshot_check", "cross_host_parts", "cross_host_product", "pick_job", "peer_label")}


def require_readable_tree(context) -> None:
    """判 Z5 之前先核冻结树认得全：快照里至少有一个文件。树不在、读不到（os.walk 走不进去）与树里没有快照要算的文件，算出来都是空集，
    空集的摘要（空输入的 sha256 e3b0c442…）不当成「摘要变了」往下判，当成运行条件不满足拒掉（第 2 次重跑登记修订 4）。"""
    tree = os.path.join(context.repo, context.tree)
    if one.row_driver_module(context).snapshot_digest(tree)[1] == 0:
        raise one.RunCondition(f"冻结树 {context.tree} 不在、读不到，或快照要算的文件（.rs 与 Cargo.*）一个都没有：快照是空集、认不全，不判 Z5")


def snapshot_check_with_mutations_table(context) -> None:
    """作废 Z5（编排器原核快照摘要）＋修订 8 第 8.6 节：树里 crates/mutations.tsv 等于 H8。核之前先要树认得全（require_readable_tree）。"""
    require_readable_tree(context)
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


def configure(config: dict, stop_file: str) -> None:
    if not broken("peer-names-revision-6"):
        one.peer_label = peer_label_of_revision_8
    STATE["mutations_sha256"] = judge.section(config, "H8").get("mutations_table_sha256")
    STATE["stop_file"] = stop_file
    one.snapshot_check = snapshot_check_with_mutations_table
    one.cross_host_parts = cross_host_parts_without_scheduling_keys
    one.cross_host_product = cross_host_product_of_revision_8
    one.pick_job = pick_job_unless_stopped


H16_ITEMS = ("run_date", "run_suffix", "dumps_root", "logs_directory")


def h16_missing_items(config: dict) -> list[str]:
    """占位 H16 里没填的几项（没有这一键、值是 null 或空串）。"""
    h16 = judge.section(config, "H16")
    return [name for name in H16_ITEMS if h16.get(name) in (None, "")]


def require_h16(config: dict) -> None:
    """H16 缺一项就拒、点名缺哪几项（第 2 次重跑登记修订 4）：不把没填的值当成字面的「None」，日志目录也就不会落进当前目录。"""
    missing = h16_missing_items(config)
    if missing:
        raise judge.RunConditionError(f"占位 H16 缺 {','.join(missing)}：起跑之前由主 agent 照修订 8 第 8.8 节填上")


def context_of(config: dict, repo: str, baseline_timing: str):
    require_h16(config)
    h16 = judge.section(config, "H16")
    logs = str(h16.get("logs_directory"))
    arguments = argparse.Namespace(repo=repo, prefix=placeholders.product_prefix(str(h16.get("run_date"))), suffix=str(h16.get("run_suffix")), resume_tag="",
                                   dumps=str(h16.get("dumps_root")), tree=judge.tree_of(config),
                                   snapshot_digest=str(judge.section(config, "H3").get("snapshot_digest")), baseline_timing=baseline_timing,
                                   check=os.path.join(HERE, "check_product.py"), logs=logs, work_root=os.path.join(logs, "rows-work"), research_root=None)
    return one.context_from_arguments(arguments)


def main(argument_list: list[str]) -> int:
    if refuse_break_switches_outside_selftest():
        return 2
    if argument_list[:1] == ["--selftest"]:
        return selftest(argument_list[1:])
    parser = argparse.ArgumentParser(description="E171 修订 8 那一趟：第 11–13 步的多机编排（第一段编排器外面一层）")
    parser.add_argument("mode", choices=("run", "plan"))
    for name in ("--config", "--baseline-timing", "--stop-file"):
        parser.add_argument(name, required=True)
    parser.add_argument("--repo", default=os.getcwd())
    arguments = parser.parse_args(argument_list)
    if arguments.mode == "run":
        phase_exit = one.shard_phase_before_layer(os.path.abspath(arguments.repo), say=lambda line: print(line, flush=True))
        if phase_exit is not None:
            return phase_exit
    try:
        config = judge.load_config(arguments.config)
        require_h16(config)   # 在 configure 之前：外面一层（修订 11）的 configure 也读 H16 的导出根
    except judge.RunConditionError as error:
        print(f"✗ {error}", file=sys.stderr)
        print('     → 怎么办：照上面那句补齐运行条件（配置、冻结树、设备文件）再用同一条命令起', file=sys.stderr)
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
        one.say(context, "收到停止信号：在跑的各行已停、别的台已做完的行已收进来；同一条命令再起就是续跑")
        return one.EXIT_STOPPED_BY_SIGNAL


# ── 自证：借第一段编排器的假世界（fake_world：假驱动、假 task-server 分片、假机器清单与资源），起的是这一层（经假 task-server 起）──
FAKE_RUN_DATE = "2026-10-09"


def fake_config(root: str, world, peer_names: tuple[str, str, str] | None = None) -> str:
    tree = os.path.join(world.repo, "scripts/experiment-apparatus/target/fake-frozen-tree")
    judge.write_text(os.path.join(tree, judge.MUTATIONS_TABLE), "# 假的坏实现表\n")
    names = peer_names or ("research/e171-s1-final-peer-kit", "research/e171-s1-final-frozen-snapshot", "scripts/experiment-apparatus/target/e171-s1-final-peer-out")
    config = {"tree": "scripts/experiment-apparatus/target/fake-frozen-tree", "H3": {"snapshot_digest": judge.row_driver.snapshot_digest(tree)[0]},
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
    label = one.peer_label(types.SimpleNamespace(tag="", attempt_dir="/x/attempt-1"), "s12-row-0-h2")
    good = label.startswith(PEER_LABEL_PREFIX) and one.snapshot_check is snapshot_check_with_mutations_table \
        and one.cross_host_parts is cross_host_parts_without_scheduling_keys
    return good, f"标签 {label}"


def cell_mutations_table_checked(scratch: str) -> tuple[bool, str]:
    world = one.fake_world(os.path.join(scratch, "world"))
    config_path = fake_config(scratch, world)
    config = judge.load_config(config_path)
    configure(config, os.path.join(scratch, "stop"))
    context = context_of(config, world.repo, "unused")
    one.snapshot_check(context)
    judge.write_text(os.path.join(world.repo, "scripts/experiment-apparatus/target/fake-frozen-tree", judge.MUTATIONS_TABLE), "# 拍完又改了\n")
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
    completed = subprocess.run(one.served_command(world.root, wrapper_command(world, config_path, stop_file)), cwd=world.repo, env=environment,
                               capture_output=True, text=True, timeout=timeout)
    return completed.returncode, completed.stdout + completed.stderr


def ledger_text(world) -> str:
    path = os.path.join(world.repo, world.dumps, "segment1-rows-shards", "row-hosts.tsv")
    return open(path, encoding="utf-8").read() if os.path.exists(path) else ""


def cell_e2e_two_hosts(scratch: str) -> tuple[bool, str]:
    world = one.fake_world(os.path.join(scratch, "world"))
    config_path = fake_config(scratch, world)
    code, output = run_world(world, config_path, os.path.join(scratch, "stop"))
    verdict = one.verdict_of(product_path(world, one.CROSS_HOST_CHECK_STEM))
    peer_outputs = [root for root, directories, _ in os.walk(os.path.join(world.repo, world.dumps, "segment1-rows-shards"))
                    if os.path.basename(root) == one.PEER_OUT_DIRECTORY_NAME]
    labels = one.relay_labels(world.root)
    good = (code == 0 and os.path.exists(product_path(world, "rows-driver")) and os.path.exists(product_path(world, "rows-probe-driver"))
            and "cross_host_identical=true" in verdict and "\tpeer\n" in ledger_text(world) and bool(peer_outputs)
            and bool(labels) and all(label.startswith(PEER_LABEL_PREFIX) for label in labels)
            and "cross_host_o3_check_sha256=" in open(product_path(world, one.CROSS_HOST_CHECK_STEM), encoding="utf-8").read())
    return good, (f"退 {code}；跨机 {verdict[:70]}；别的台写在共享根下的分片输出目录 {len(peer_outputs)}；别的台标签 {sorted(set(label[:8] for label in labels))}；"
                  f"末行 {output.strip().splitlines()[-1:]}")


def cell_e2e_stop_file(scratch: str) -> tuple[bool, str]:
    world = one.fake_world(os.path.join(scratch, "world"))
    config_path = fake_config(scratch, world)
    stop_file = judge.write_text(os.path.join(scratch, "stop"), "单元 step-3-o2 红\n")
    code, output = run_world(world, config_path, stop_file)
    ran = sum(1 for line in open(os.path.join(world.root, "driver.log"), encoding="utf-8") if line.startswith("ran ")) \
        if os.path.exists(os.path.join(world.root, "driver.log")) else 0
    return code == 1 and ran == 0, f"有停派文件：退 {code}，真跑过的行 {ran}（要 0）"


def pinned_selftest_cell(path: str, last_line: str) -> tuple[bool, str]:
    environment = {key: value for key, value in os.environ.items() if not key.startswith(("ROWS_R8_", "ROWS_ORCHESTRATOR_", "CROSS_HOST_O3_"))}
    completed = subprocess.run([sys.executable, "-B", path, "--selftest"], capture_output=True, text=True, env=environment, timeout=900)
    got = completed.stdout.strip().splitlines()[-1:] or [""]
    return completed.returncode == 0 and got[0] == last_line, f"{os.path.basename(path)} --selftest 末行 {got[0]}（要 {last_line}）"


SELFTEST_CELLS = {
    "overrides-applied": cell_overrides_applied, "mutations-table-checked": cell_mutations_table_checked,
    "cross-host-o3-scheduling-keys": cell_cross_host_o3_scheduling_keys, "e2e-two-hosts": cell_e2e_two_hosts, "e2e-stop-file": cell_e2e_stop_file,
    "segment-one-orchestrator-selftest": lambda scratch: pinned_selftest_cell(SEGMENT_ONE_ORCHESTRATOR_PATH, SEGMENT_ONE_SELFTEST_LAST_LINE),
    "cross-host-check-selftest": lambda scratch: pinned_selftest_cell(CROSS_HOST_CHECK_PATH, CROSS_HOST_SELFTEST_LAST_LINE),
}
SELFTEST_SWITCHES = {"peer-names-revision-6": "overrides-applied", "mutations-table-skipped": "mutations-table-checked",
                     "cross-host-original-o3": "cross-host-o3-scheduling-keys", "stop-file-ignored": "e2e-stop-file"}


def selftest(argument_list: list[str]) -> int:
    os.environ[SELFTEST_VARIABLE] = "1"
    return harness.run_selftest(os.path.abspath(__file__), argument_list, SELFTEST_CELLS, SELFTEST_SWITCHES, BREAK_VARIABLE, SELFTEST_VARIABLE)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
