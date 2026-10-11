#!/usr/bin/env python3
"""E171 第 1 次重跑·修订 10：修订 8 那一趟第二段整表的多机编排（下称「第二段·修订 8 编排层」；登记 research/prompts/e171-r1-prereg.md
修订 10 第 10.2、10.3、10.7、10.9 节与「主 agent 认定：修订 10 交回」）。由 run_segment_two_revision_10_multi_host.sh 起，在仓根跑；
值从 revision_10_placeholders.json 那一形的配置与它点名的修订 8 那一份配置、修订 8 那一趟的序列日志读（revision_10_placeholders.py）。

为什么另写一层：rows_orchestrator_segment_two.py（132731f8…）与入口 run_segment_two_on_the_revision_6_tree_multi_host.sh（c582ee61…）一个字节不改
（修订 6 那一趟的输入清单记着前者；入口里树、摘要、T、变异表写死成修订 6 树的值）。这一层按路径载入 rows_orchestrator_segment_two.py（开跑先核 sha256，
不是就退 3；它照旧按 sha256 载入第一段编排器 7bfbdb84…），行集、驱动参数、分配、补位协调、合并规则 ①–⑦、判法 check_product.py rows_without_executor、
跨机对照的取法（权重最接近 1 的那一行，现算）与比法、超时重跑只留档、续跑都调它原来的函数。改的只有修订 10 第 10.2 节「这一趟改的几处」与第 10.3 节：
  ① 树、快照摘要（H3）、树里变异表（H8）、T₂ 那份计时产物（H21）从修订 8 那一份配置与序列日志读；产物前缀 …-<RUN_DATE>-s2、后缀 -r8、导出根、日志目录、
    工作根取 H23；
  ② 每行前后（编排器起驱动之前、收尾之后核 Z5 的那两处，与开跑、收尾各一次）另核树里 crates/mutations.tsv 等于 H8，不等记作废 Z5、停（退 1）；
  ③ 别的台四个落点（工具包、冻结树快照、取回、起法目录）与标签前缀取 H24（不与修订 6 第二段、修订 8 第一段同名）；
  ④ 起跑闸：修订 8 那一趟第 0、1、2 步（与研究一侧变异表，排了的话）绿，没过退 78；T₂ 包不住各流之和退 78（第 10.1 节：不起、交主 agent）；
  ⑤ 本机闸：第 7 步绿、修订 8 的逐行编排起过之前，分配那一步把本机的槽数记 0；之后经编排器原有的「起跑时放不下的台每 600 秒重查、放得下就加槽」
    给本机加槽（已有槽的台不增不减，第 10.2 节表第 6 条）；
  ⑥ 停派文件：<H16 日志目录>/stop-dispatching-rows 在时不再派新行，在跑的跑完再停（退 1）；已落的行原样留着，主 agent 处置之后同一条命令续跑；
  ⑦ 让路（H27）：修订 8 的逐行编排在跑、第 11、12 步与跨机对照 X 还有既没做完也没在跑的行时，第二段在各台都不起新行，在跑的跑完；
    第一段逐行的槽由它自己的加槽（rows_orchestrator_revision_8.py 每 600 秒重查）拿走。第 13 步超时重跑那几行不看。让路只动挂钟，不动任何读数；
  ⑧ 输入清单（H26）：第一趟开跑那一刻写 <导出根>/inputs.sha256（研究一侧文件、T₂ 那份计时产物、树里的变异表）与 run-values.tsv（树、H3、H8、T₂、
    前缀后缀、落点）；续跑先核，对不上停（退 1）：第 1 步在第二段起了之后又重跑（新的 -attempt<N>），T₂ 换了来源，已跑的行不与新 T₂ 下跑的行合进一份产物；
  ⑨ S17 两处联动（第 10.7 节）：第一段那一处（<H16 前缀>-rows-cross-host-check<H16 后缀>.out 的 verdict 行）cross_host_identical=false 时，
    两段在别的台跑的行一起不作数：这一层同停派文件一样不再派新行、在跑的跑完再停（退 1），交主 agent；第二段那一处
    （<H23 前缀>-rows-cross-host-check-r8.out）不同时编排器原来就停（退 1）。修订 8 的逐行编排不看第二段那一处（修订 8 的装置这一次不改），
    那一边由主 agent 照第 10.7 节处置。
派发那一个循环照抄 rows_orchestrator_segment_two.py 的 run_queue_two（第 629–685 行），改两处：⑥ ⑨ 不再派新行的缘由、⑦ 让路。

用法：
  rows_orchestrator_segment_two_revision_10.py run  --config C [--repo R]
      跑。先照 revision_10_placeholders.py check --phase start 核两份配置（不过退 78）、核起跑闸与 T₂，再写或核输入清单，再交编排器。
      被停、被杀之后同一条命令再起就是续跑（只补行目录里还没有 row-result.tsv 的行）。
  rows_orchestrator_segment_two_revision_10.py plan --config C [--repo R]
      干跑（PLAN_ONLY）：占位核对（不过只报）、起跑闸、本机闸、让路、停派文件、T₂、现查的分配与推的各台挂钟、每一行驱动的命令；
      一行都不起、不编、不写产物、导出根、日志与输入清单。
  rows_orchestrator_segment_two_revision_10.py --selftest [--cells 格,…] [--rounds N]
退出码：0 跑完；1 停机或某一行没跑成、停派文件、S17（两处任一）、输入清单对不上；2 用法错；3 rows_orchestrator_segment_two.py 不是钉的那一份；
  78 运行条件不满足（占位没填或核不过、起跑闸没过、T₂ 取不到或包不住、多机配置、资源、预检、此刻放不下）；143 收到 TERM / INT / HUP。

自证只换假件的变量（只在 SEGMENT_TWO_R10_SELFTEST=1 时认）：SEGMENT_TWO_R10_SKIP_CONFIG_CHECK=1（假世界里不核两份配置，起跑闸与 T₂ 照核）、
  SEGMENT_TWO_R10_YIELD_RECHECK_SECONDS（让路的重查间隔，平常 30 秒）、SEGMENT_TWO_R10_RESEARCH_ROOT（研究一侧文件在哪棵仓里，平常是 --repo）。
弄坏开关 SEGMENT_TWO_R10_BREAK=<项>（逗号连几个；只在 --selftest 起的子进程里认）。项名、弄坏哪一处、打开后红在哪一格：
  peer-names-unchanged      别的台的落点与标签不换成 H24 的                        overrides-applied
  mutations-table-skipped   每行前后不核树里 crates/mutations.tsv                  mutations-table-checked
  local-gate-ignored        本机闸没开也给本机分槽                                  e2e-local-gate-closed
  running-shards-ignored    让路时第一段在跑的行也当排着的                          segment-one-rows-waiting
  yield-ignored             第一段逐行还有排着的行也照样派                          e2e-yield
  stop-file-ignored         见到停派文件照样派新行                                  e2e-stop-file
  s17-ignored               第一段那一处 S17 触发了照样派新行                       e2e-s17-segment-one
  gate-ignored              起跑闸没过也起                                          e2e-gate-refused
  inputs-unchecked          续跑不核 inputs.sha256                                  inputs-checked
  run-values-unchecked      续跑不核 run-values.tsv                                 run-values-checked
  plan-writes-inputs        干跑也写输入清单                                        plan-writes-nothing
  sweep-skipped、temporary-kept（自证外壳 revision_8_selftest_harness.py 的两个）     no-leftover-processes、temporary-directory-removed
"""
from __future__ import annotations

import argparse
import dataclasses
import hashlib
import importlib.util
import json
import os
import re
import shlex
import subprocess
import sys
import time
import types

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import frozen_tree_lines_revision_8 as judge  # noqa: E402
import revision_10_placeholders as placeholders  # noqa: E402
import revision_8_selftest_harness as harness  # noqa: E402

BREAK_VARIABLE = "SEGMENT_TWO_R10_BREAK"
SELFTEST_VARIABLE = "SEGMENT_TWO_R10_SELFTEST"
SEGMENT_TWO_ORCHESTRATOR_PATH = os.path.join(HERE, "rows_orchestrator_segment_two.py")
SEGMENT_TWO_ORCHESTRATOR_SHA256 = "132731f83e0582e80cfff6711299375569e6446958c027ca9c61b27df7a839a1"
SEGMENT_TWO_SELFTEST_LAST_LINE = "自证 46 项，红 0"
EXIT_WRONG_PINNED_FILE = 3
YIELD_RECHECK_SECONDS = 30.0
# 修订 8 那一趟第 11、12 步与跨机对照 X（第一段编排器 STEPS 的键）：让路看它们还有没有排着的行
SEGMENT_ONE_ROW_STEPS = ("s11", "s12", "sx")
# 输入清单（H26）里的研究一侧文件：修订 6 第二段那 13 个里的 11 个（基线计时换成 T₂ 那一份、树里的变异表另列），加这一层与它 import 的几份
INPUT_RESEARCH_FILES = ("row_driver.py", "device_timing.py", "frozen_o3.py", "run_directory.py", "bad_implementation_table.py", "bad-implementations-117.tsv",
                        "bad-implementations-expansion-164.tsv", "rows_orchestrator.py", "rows_orchestrator_segment_two.py", "peer_side.py", "check_product.py",
                        "rows_orchestrator_segment_two_revision_10.py", "revision_10_placeholders.py", "run_segment_two_revision_10_multi_host.sh",
                        "revision_8_selftest_harness.py", "revision_8_placeholders.py", "frozen_tree_lines_revision_8.py", "segment_one_units_revision_8.py")


class WrongPinnedFile(Exception):
    """rows_orchestrator_segment_two.py 的 sha256 不是钉的那一份。"""


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
    two = load_pinned(SEGMENT_TWO_ORCHESTRATOR_PATH, SEGMENT_TWO_ORCHESTRATOR_SHA256, "rows_orchestrator_segment_two_132731f8")
except WrongPinnedFile as wrong_file:
    print(f"✗ {wrong_file}（这一层调的是钉住的那一份的函数；改过就先核这一层，再改钉的 sha256）", file=sys.stderr)
    sys.exit(EXIT_WRONG_PINNED_FILE)
one = two.one


def broken(item: str) -> bool:
    return harness.broken(BREAK_VARIABLE, item)


def selftest_override(name: str) -> str:
    """只在自证起的子进程里认的换假件变量；平常交回空串。"""
    return os.environ.get(name, "") if os.environ.get(SELFTEST_VARIABLE) == "1" else ""


def yield_recheck_seconds() -> float:
    return float(selftest_override("SEGMENT_TWO_R10_YIELD_RECHECK_SECONDS") or YIELD_RECHECK_SECONDS)


# ── 换进编排器的几处（configure 一次装好；编排器里的函数按模块全局名取，换了就用这里的） ──
STATE: dict = {"segment_one": None, "tree_table_sha256_h8": None, "label_prefix": None, "yield_checked_at": float("-inf"), "yield_waiting": []}
ORIGINAL = {"snapshot_check": one.snapshot_check, "allocate": one.allocate}


def snapshot_check_with_the_tree_table(context) -> None:
    """作废 Z5：编排器原来核快照摘要（H3）；修订 10 第 10.2 节另核树里 crates/mutations.tsv 等于 H8。"""
    ORIGINAL["snapshot_check"](context)
    if broken("mutations-table-skipped"):
        return
    found = judge.sha256_or_missing(os.path.join(context.repo, context.tree, judge.MUTATIONS_TABLE))
    if found != STATE["tree_table_sha256_h8"]:
        raise one.Stopped(f"树里 crates/mutations.tsv 不是 H8 那一份（作废 Z5）：现算 {found}")


def allocate_behind_the_local_gate(hosts, resources):
    """编排器的分配（文件头 ⑤）：本机闸没开时本机那一台的槽数记 0，别的台照编排器原来的算法。"""
    allocations = ORIGINAL["allocate"](hosts, resources)
    gate_open, why = placeholders.local_gate(STATE["segment_one"])
    if gate_open or broken("local-gate-ignored"):
        return allocations
    return [dataclasses.replace(allocation, slots=0, threads_per_slot=0, terms=f"本机闸没开（{why}），本机槽数记 0")
            if allocation.number == 1 else allocation for allocation in allocations]


def peer_label_of_this_segment(context, label: str) -> str:
    """别的台那一趟的标签（预检与每一行都用它）：H24 的标签前缀接趟名与分片名。"""
    return re.sub(r"[^A-Za-z0-9._-]", "-", f"{STATE['label_prefix']}-{os.path.basename(context.attempt_dir)}-{label}")


def configure(segment_one: placeholders.SegmentOne, segment_two: placeholders.SegmentTwo) -> None:
    if not broken("peer-names-unchanged"):
        one.PEER_KIT_RELATIVE, one.PEER_SNAPSHOT_RELATIVE, one.PEER_OUT_RELATIVE, two.PEER_LAUNCH_RELATIVE = segment_two.peer_names
        one.peer_label = peer_label_of_this_segment
        two.peer_label_two = peer_label_of_this_segment
    STATE.update(segment_one=segment_one, tree_table_sha256_h8=segment_one.mutations_sha256, label_prefix=segment_two.label_prefix,
                 yield_checked_at=float("-inf"), yield_waiting=[])
    one.snapshot_check = snapshot_check_with_the_tree_table
    one.allocate = allocate_behind_the_local_gate
    two.run_queue_two = run_queue_with_stop_file_and_yield


# ── 让路与停派文件（文件头 ⑥ ⑦） ──
def running_segment_one_rows(shards: str) -> set[tuple[str, int]]:
    """修订 8 逐行编排最近一趟（shards 下编号最大的 attempt-<N>）里起了、还没写 exit 的分片的 (步, 行)；重报的 carry 分片不算。"""
    if broken("running-shards-ignored") or not os.path.isdir(shards):
        return set()
    attempts = sorted((int(name.split("-", 1)[1]) for name in os.listdir(shards) if re.fullmatch(r"attempt-[0-9]+", name)), reverse=True)
    if not attempts:
        return set()
    latest = os.path.join(shards, f"attempt-{attempts[0]}")
    found = set()
    for entry in sorted(os.listdir(latest)):
        record_path = os.path.join(latest, entry, "shard.json")
        if not os.path.isfile(record_path) or os.path.exists(os.path.join(latest, entry, "exit")):
            continue
        record = json.load(open(record_path, encoding="utf-8"))
        if record.get("kind") in ("local", "peer"):
            found |= {(str(record.get("step")), int(row)) for row in record.get("rows", [])}
    return found


def segment_one_rows_waiting(repo: str, segment_one: placeholders.SegmentOne) -> list[str]:
    """修订 8 的逐行编排在跑时，第 11、12 步与 X 里既没做完（行目录没有 row-result.tsv）、也没在跑的行（「<步>:<行>」）；它没在跑交回空表。"""
    if not placeholders.rows_orchestrator_running(placeholders.sequence_lines(segment_one)):
        return []
    rows_root = os.path.join(judge.resolve(repo, segment_one.dumps), "segment1-rows")
    running = running_segment_one_rows(rows_root + "-shards")
    waiting = []
    for step in SEGMENT_ONE_ROW_STEPS:
        root = rows_root + one.STEPS[step].root_suffix
        for row in one.STEP_ROWS[step]:
            if not os.path.exists(os.path.join(root, f"row-{row}", "row-result.tsv")) and (step, row) not in running:
                waiting.append(f"{step}:{row}")
    return waiting


def yielding(context) -> bool:
    """让路：每 yield_recheck_seconds() 秒重查一次第一段逐行还有没有排着的行；进出让路各记一行日志。"""
    if broken("yield-ignored"):
        return False
    if time.monotonic() - STATE["yield_checked_at"] >= yield_recheck_seconds():
        STATE["yield_checked_at"] = time.monotonic()
        waiting = segment_one_rows_waiting(context.repo, STATE["segment_one"])
        if bool(waiting) != bool(STATE["yield_waiting"]):
            one.say(context, f"  让路：第一段逐行还有 {len(waiting)} 行排着（{','.join(waiting[:8])}），第二段不起新行，在跑的跑完" if waiting
                    else "  让路结束：第一段逐行没有排着的行了，第二段接着派")
        STATE["yield_waiting"] = waiting
    return bool(STATE["yield_waiting"])


def stop_reason(repo: str) -> str | None:
    """不再派新行的两种缘由（文件头 ⑥ ⑨），交回一句说明；都没有交回 None。"""
    segment_one = STATE["segment_one"]
    if os.path.exists(segment_one.stop_file) and not broken("stop-file-ignored"):
        return f"见到停派文件 {segment_one.stop_file}（修订 8 那一趟第 0–10 步有单元红，第 10.3 节第 4 条）：不再派新行，在跑的跑完再停"
    product, identical = placeholders.segment_one_cross_host_verdict(repo, segment_one)
    if identical == "false" and not broken("s17-ignored"):
        return (f"S17 在第一段那一处触发（{product} 的 verdict 行 cross_host_identical=false）：两段在别的台跑的行一起不作数（第 10.7 节），"
                "不再派新行，在跑的跑完再停，交主 agent")
    return None


def run_queue_with_stop_file_and_yield(context, slots: list[tuple[int, int, int]], jobs: list, expected_hashes: dict[str, str], setup,
                                       regrow=None, avoid_host: int | None = None) -> tuple[list[dict], list, list[str]]:
    """rows_orchestrator_segment_two.py 的 run_queue_two 照抄，改两处：见到停派文件记一条没跑成的说明（不再派新行，在跑的跑完）；
    让路时这一轮不派。交回 (跑成的分片记录, 没派出去的, 没跑成的说明)。"""
    free = [slot for slot in slots if slot[0] != avoid_host]
    queue = two.queue_order_two(jobs)
    running: list = []
    succeeded: list[dict] = []
    failures: list[str] = []
    next_regrow = time.monotonic() + two.regrow_interval_seconds()
    stop_noted = False
    while not one.STOP.is_set():
        reason = None if stop_noted else stop_reason(context.repo)
        if reason:
            stop_noted = True
            failures.append(reason)
            one.say(context, "  " + reason)
        if regrow is not None and queue and not failures and time.monotonic() >= next_regrow:
            next_regrow = time.monotonic() + two.regrow_interval_seconds()
            free.extend(slot for slot in regrow() if slot[0] != avoid_host)
        dispatching = not failures and bool(queue) and not yielding(context)
        for slot in sorted(free) if dispatching else []:
            if not queue:
                break
            job = queue[0]
            step = two.step_of(context, job.step)
            label = two.unique_label(context, f"{job.step}-row-{job.row}-h{slot[0]}")
            try:
                one.snapshot_check(context)  # 作废 Z5（另核 H8）：每个驱动起之前核一次
                exclusion_names = two.exclusions_for(context, job.step, job.row)
                if slot[0] == 1:
                    process, record = two.launch_local_two(context, expected_hashes, label, step, (job.row,), slot[2],
                                                           two.work_root_for(context, job, slot, label), "local", exclusion_names)
                else:
                    process, record = two.launch_peer_two(context, setup.kit, label, step, job.row, slot[2], slot[0],
                                                          setup.configurations.get(slot[0]), exclusion_names)
            except one.Stopped as error:
                failures.append(str(error))
                break
            queue.pop(0)
            free.remove(slot)
            running.append(two.RunningTwo(job, slot, process, record))
            one.say(context, f"  ▶ {job.step} 第 {job.row} 行 → {one.host_name(slot[0])} 第 {slot[1]} 槽（{slot[2]} 线程，补位排除 {len(exclusion_names)} 个名字）")
        for item in list(running):
            code = item.process.poll()
            if code is None:
                continue
            running.remove(item)
            free.append(item.slot)
            if two.finish_shard_two(context, item.record, code):
                succeeded.append(item.record)
            else:
                failures.append(f"{item.job.step} 第 {item.job.row} 行（{one.host_name(item.slot[0])}，退 {code}，见 {item.record['label']}/）")
            try:
                one.snapshot_check(context)  # 作废 Z5（另核 H8）：每个驱动收尾再核一次
            except one.Stopped as error:
                failures.append(str(error))
        if not running and (failures or not queue or not free):
            break
        time.sleep(context.poll_seconds)
    if one.STOP.is_set():
        two.stop_running_two(context, running)
        raise one.Interrupted()
    return succeeded, queue, failures


# ── 输入清单（H26，文件头 ⑧） ──
def shown_path(repo: str, path: str) -> str:
    """清单里的路径：在仓里的写相对仓根，别的写绝对路径。"""
    absolute = os.path.abspath(path)
    return os.path.relpath(absolute, repo) if absolute.startswith(repo.rstrip("/") + "/") else absolute


def input_files(repo: str, research_root: str, segment_one: placeholders.SegmentOne, timing: placeholders.BaselineTimingOfStepOne) -> list[str]:
    research = os.path.join(research_root, placeholders.RESEARCH_RELATIVE)
    return ([shown_path(repo, os.path.join(research, name)) for name in INPUT_RESEARCH_FILES]
            + [timing.path, os.path.join(segment_one.tree, judge.MUTATIONS_TABLE)])


def run_values(segment_one: placeholders.SegmentOne, segment_two: placeholders.SegmentTwo, timing: placeholders.BaselineTimingOfStepOne) -> list[tuple[str, str]]:
    return [("tree", segment_one.tree), ("snapshot_digest_h3", segment_one.snapshot_digest), ("tree_mutations_table_sha256_h8", segment_one.mutations_sha256),
            ("baseline_timing_tag_h21", timing.tag or "-"), ("baseline_timing_h21", timing.path), ("baseline_timing_sha256_h21", timing.sha256),
            ("baseline_timing_seconds_h21", f"{timing.seconds:.3f}"), ("row_time_limit_seconds", str(timing.row_time_limit_seconds)),
            ("segment_one_dumps_h16", segment_one.dumps), ("segment_one_logs_h16", segment_one.logs), ("product_prefix_h23", segment_two.prefix),
            ("product_suffix_h23", segment_two.suffix), ("peer_names_h24", ",".join(segment_two.peer_names)), ("label_prefix_h24", segment_two.label_prefix)]


def inputs_text(repo: str, files: list[str]) -> str:
    return "".join(f"{judge.sha256_or_missing(judge.resolve(repo, path))}  {path}\n" for path in files)


def values_text(values: list[tuple[str, str]]) -> str:
    return "".join(f"{key}\t{value}\n" for key, value in values)


def record_or_check_inputs(repo: str, dumps: str, files: list[str], values: list[tuple[str, str]]) -> str:
    """第一趟：建导出根，写 inputs.sha256 与 run-values.tsv（排他新建）；续跑：两份逐行核，对不上抛 Stopped。交回一句给日志的话。"""
    root = judge.resolve(repo, dumps)
    inputs_path, values_path = os.path.join(root, placeholders.INPUTS_FILE), os.path.join(root, placeholders.RUN_VALUES_FILE)
    if not os.path.exists(values_path):
        os.makedirs(root, exist_ok=True)
        for path, text in ((inputs_path, inputs_text(repo, files)), (values_path, values_text(values))):
            with open(path, "x", encoding="utf-8") as handle:
                handle.write(text)
        return f"输入清单记下：{shown_path(repo, inputs_path)}（{len(files)} 个文件）与 {placeholders.RUN_VALUES_FILE}（{len(values)} 项）"
    problems = []
    if not broken("inputs-unchecked"):
        recorded = [line.split("  ", 1) for line in open(inputs_path, encoding="utf-8").read().splitlines()] if os.path.exists(inputs_path) else []
        problems += [f"{path}（记的 {digest[:12]}，现在 {judge.sha256_or_missing(judge.resolve(repo, path))[:12]}）" for digest, path in recorded
                     if judge.sha256_or_missing(judge.resolve(repo, path)) != digest]
        if [path for _, path in recorded] != files:
            problems.append(f"清单里的文件不是这一份配置推出来的那 {len(files)} 个")
    if not broken("run-values-unchecked"):
        recorded_values = dict(line.split("\t", 1) for line in open(values_path, encoding="utf-8").read().splitlines() if "\t" in line)
        problems += [f"{key}（记的 {recorded_values.get(key, '-')}，现在 {value}）" for key, value in values if recorded_values.get(key) != value]
    if problems:
        raise one.Stopped("输入清单对不上第一趟记下的（H26；第 1 步重跑过就是 T₂ 换了来源，已跑的行不与新 T₂ 下跑的合进一份，交主 agent）："
                          + "；".join(problems[:6]))
    return f"输入清单与第一趟相同（{len(files)} 个文件、{len(values)} 项）"


# ── 主流程 ──
def research_root_of(repo: str) -> str:
    return os.path.abspath(selftest_override("SEGMENT_TWO_R10_RESEARCH_ROOT") or repo)


def context_of(repo: str, segment_one: placeholders.SegmentOne, segment_two: placeholders.SegmentTwo, timing: placeholders.BaselineTimingOfStepOne):
    arguments = argparse.Namespace(repo=repo, prefix=segment_two.prefix, suffix=segment_two.suffix, dumps=segment_two.dumps, tree=segment_one.tree,
                                   snapshot_digest=segment_one.snapshot_digest, baseline_timing=timing.path, check=os.path.join(HERE, "check_product.py"),
                                   logs=segment_two.logs, work_root=segment_two.work_root, research_root=research_root_of(repo))
    return two.context_from_arguments_two(arguments)


def row_command(context, row: int, threads: str) -> list[str]:
    """干跑打的一行本机驱动命令（launch_local_two 那一份；工作根、分片目录、排除表跑的时候才定，这里写成尖括号）。"""
    return (["nice", "-n", "19", "python3", "-B", context.driver_path, "--repo-root", context.repo, "--work-root", f"{context.work_root}/<槽>",
             "--output-root", context.step_root(two.ROWS_STEP), "--snapshot-from", os.path.join(context.repo, context.tree),
             "--baseline-timing", os.path.join(context.repo, context.baseline_timing), "--timing-output", "<分片目录>/timing.out"]
            + two.driver_arguments(two.ROWS_STEP, (row,), threads, "<分片目录>/send/exclusions.tsv"))


def plan_text(repo: str, segment_one: placeholders.SegmentOne, segment_two: placeholders.SegmentTwo,
              timing: placeholders.BaselineTimingOfStepOne | None) -> list[str]:
    gate = placeholders.start_gate_problems(repo, segment_one)
    local_open, local_why = placeholders.local_gate(segment_one)
    waiting = segment_one_rows_waiting(repo, segment_one)
    lines = [f"依赖·起跑闸（修订 8 第 0、1、2 步与研究一侧变异表，排了的话）：{'过' if not gate else '没过：' + ','.join(gate)}",
             f"依赖·本机闸：{'开' if local_open else '没开'}（{local_why}）",
             f"依赖·让路：第一段逐行排着的 {len(waiting)} 行{('（' + ','.join(waiting[:8]) + '）') if waiting else ''}",
             f"依赖·停派文件 {segment_one.stop_file}：{'在' if os.path.exists(segment_one.stop_file) else '不在'}",
             "依赖·S17 第一段那一处 {0}：cross_host_identical={1}".format(*placeholders.segment_one_cross_host_verdict(repo, segment_one)),
             f"产物前缀 {segment_two.prefix}-…{segment_two.suffix}.out；导出根 {segment_two.dumps}；日志 {segment_two.logs}；工作根 {segment_two.work_root}",
             f"别的台落点 {','.join(segment_two.peer_names)}；标签前缀 {segment_two.label_prefix}"]
    if timing is not None:
        lines.append(f"T₂：TAG {timing.tag or '-'}，{timing.path}（sha256 {timing.sha256}），process_wall {timing.seconds:.3f} 秒，各流之和 "
                     f"{timing.stream_seconds:.3f} 秒，逐行限时 {timing.row_time_limit_seconds} 秒")
    return lines


def print_row_commands(context) -> None:
    """干跑：每一行本机驱动那一条命令（按派发的次序：推的用时最长先），再打别的台那一行的起法（multi-host-run.sh --peer 那一层）。"""
    ordered = sorted(context.rows, key=lambda row: (-context.row_seconds.get(row, 0.0), row))
    for row in ordered:
        print(f"行 {row}（推 {context.row_seconds.get(row, 0.0):.0f} 秒）：{shlex.join(row_command(context, row, '<每槽线程>'))}")
    peer_inner = (["run-driver", "--expected", f"{one.PEER_KIT_RELATIVE}/expected.json", "--"]
                  + two.driver_arguments(two.ROWS_STEP, (ordered[0],), "<每槽线程>", f"{two.PEER_LAUNCH_RELATIVE}/{two.EXCLUSIONS_FILE}"))
    peer = (["bash", context.multi_host_run, "--peer", "--memory-cap", one.PEER_OUTER_MEMORY_CAP, "--label", f"{STATE['label_prefix']}-attempt-<趟>-rows-row-{ordered[0]}-h<台>",
             "--send", "<工具包>", one.PEER_KIT_RELATIVE, "--send", "<冻结树快照>", one.PEER_SNAPSHOT_RELATIVE, "--send", "<分片目录>/send", two.PEER_LAUNCH_RELATIVE,
             "--fetch", one.PEER_OUT_RELATIVE, "--", "python3", "-B", f"{one.PEER_KIT_RELATIVE}/peer_side.py"] + peer_inner)
    print(f"别的台那一行（第 {ordered[0]} 行为例）：{shlex.join(peer)}")
    print(f"行命令 {len(ordered)} 条；跨机对照那一行第 {context.cross_host_row} 行在整表那一次之外的另一台再跑一次（同一张补位排除表）")


def run_layer(repo: str, config_path: str, plan_only: bool) -> int:
    try:
        config, revision_8_config = placeholders.load_configs(repo, config_path)
    except judge.RunConditionError as error:
        print(f"✗ {error}（退 78）", flush=True)
        return one.EXIT_RUN_CONDITION
    segment_one, segment_two = placeholders.segment_one_of(revision_8_config), placeholders.segment_two_of(config)
    if selftest_override("SEGMENT_TWO_R10_SKIP_CONFIG_CHECK") != "1":
        unfilled_keys, failing, printed = placeholders.check_config(repo, config, revision_8_config, "start")
        for line in printed:
            print(line, flush=True)
        print(f"name=placeholders_verdict phase=start config={config_path} unfilled={','.join(unfilled_keys) or 'none'} failing={','.join(failing) or 'none'}", flush=True)
        if (unfilled_keys or failing) and not plan_only:
            print("✗ 两份配置有没填或核不过的（见上面 filled=false、holds=false 那几行）：不起（退 78）", flush=True)
            return one.EXIT_RUN_CONDITION
    gate = placeholders.start_gate_problems(repo, segment_one)
    if gate and not plan_only and not broken("gate-ignored"):
        print(f"✗ 起跑闸没过（{','.join(gate)}）：修订 8 那一趟第 0、1、2 步（与研究一侧变异表，排了的话）绿了再用同一条命令起（退 78）", flush=True)
        return one.EXIT_RUN_CONDITION
    try:
        timing = placeholders.resolve_baseline_timing(repo, segment_one)
    except judge.RunConditionError as error:
        if plan_only:
            for line in plan_text(repo, segment_one, segment_two, None):
                print(line)
        print(f"✗ {error}（退 78）", flush=True)
        return one.EXIT_RUN_CONDITION
    configure(segment_one, segment_two)
    context = context_of(repo, segment_one, segment_two, timing)
    files, values = input_files(repo, research_root_of(repo), segment_one, timing), run_values(segment_one, segment_two, timing)
    try:
        if plan_only:
            for line in plan_text(repo, segment_one, segment_two, timing):
                print(line)
            if broken("plan-writes-inputs"):
                record_or_check_inputs(repo, segment_two.dumps, files, values)
            code = two.run_two(context, True)
            print_row_commands(context)
            print(f"输入清单（H26，第一趟开跑那一刻才写）会记 {len(files)} 个文件与 {len(values)} 项：" + "，".join(files[-2:]) + " 等")
            return code
        one.say(context, f"第二段·修订 8 编排层开跑：配置 {config_path}（sha256 {judge.sha256_of_file(config_path)[:16]}…），树 {segment_one.tree}，"
                         f"产物 {segment_two.prefix}-*{segment_two.suffix}.out，导出 {segment_two.dumps}")
        for line in plan_text(repo, segment_one, segment_two, timing):
            one.say(context, "  " + line)
        one.say(context, "  " + record_or_check_inputs(repo, segment_two.dumps, files, values))
        return two.run_two(context, False)
    except one.RunCondition as error:
        one.say(context, f"✗ 运行条件不满足（退 78）：{error}")
        return one.EXIT_RUN_CONDITION
    except one.Stopped as error:
        one.say(context, f"✗ 停：{error}")
        return 1
    except one.Interrupted:
        one.say(context, "收到停止信号：在跑的各行已停、别的台已做完的行已取回收进来；同一条命令再起就是续跑")
        return one.EXIT_STOPPED_BY_SIGNAL


def main(argument_list: list[str]) -> int:
    for variable, selftest_variable in ((BREAK_VARIABLE, SELFTEST_VARIABLE), (two.BREAK_VARIABLE, two.SELFTEST_VARIABLE),
                                        ("ROWS_ORCHESTRATOR_BREAK", "ROWS_ORCHESTRATOR_SELFTEST")):
        refusal = harness.refuse_break_outside_selftest(variable, selftest_variable)
        if refusal:
            print(refusal, file=sys.stderr)
            return 2
    if os.environ.get(two.SELFTEST_ROWS_VARIABLE) and os.environ.get(two.SELFTEST_VARIABLE) != "1":
        print(f"✗ {two.SELFTEST_ROWS_VARIABLE} 只在自证起的子进程里认（平常跑整表 1–281）；设了它就拒（退 2）", file=sys.stderr)
        return 2
    if argument_list[:1] == ["--selftest"]:
        return selftest(argument_list[1:])
    parser = argparse.ArgumentParser(description="E171 修订 10：修订 8 那一趟第二段整表的多机编排（rows_orchestrator_segment_two.py 外面一层）")
    parser.add_argument("mode", choices=("run", "plan"))
    parser.add_argument("--config", required=True)
    parser.add_argument("--repo", default=os.getcwd())
    arguments = parser.parse_args(argument_list)
    return run_layer(os.path.abspath(arguments.repo), arguments.config, arguments.mode == "plan")


# ── 自证：借第二段编排器的假世界（fake_world_two：假驱动、假 multi-host-run、假机器清单与资源），起的是这一层；
#    另写假的两份配置、假的修订 8 序列日志与 T₂ 那份计时产物 ──
FAKE_RUN_DATE = "2026-10-09"
SEGMENT_ONE_GREEN_UNITS = {
    "gate-not-green": ("step-0", "step-1"),
    "before-step-7": ("step-0", "step-1", "step-2", "research-mutations"),
    "rows-running": ("step-0", "step-1", "step-2", "research-mutations", "step-7"),
    "finished": ("step-0", "step-1", "step-2", "research-mutations", "step-7"),
}


@dataclasses.dataclass
class WorldTen:
    base: object
    config: str
    segment_one: placeholders.SegmentOne
    segment_two: placeholders.SegmentTwo


def segment_one_lines(segment_one: placeholders.SegmentOne, repo: str, kind: str) -> list[str]:
    timing = segment_one.timing(placeholders.STEP_1_STEM)
    counts = {timing: placeholders.units.line_count(os.path.join(repo, timing))}
    lines = [placeholders.green_line(unit, [timing] if unit == "step-1" else [], counts) for unit in SEGMENT_ONE_GREEN_UNITS[kind]]
    if kind in ("rows-running", "finished"):
        lines.append(f"{FAKE_RUN_DATE} {placeholders.ROWS_ORCHESTRATOR_STARTED}（step-0, step-1, step-2, step-2b, step-7 都绿）")
    if kind == "finished":
        lines.append(f"{FAKE_RUN_DATE}   逐行编排退 0")
    return lines


def fake_world_ten(root: str, kind: str = "finished", rows: str = "1,2,3,4,5,6", resources: list[dict] | None = None,
                   extra_environment: dict[str, str] | None = None, mode: str = "run") -> WorldTen:
    base = two.fake_world_two(root, rows=rows, resources=resources, extra_environment=extra_environment)
    tree = os.path.join(base.repo, "scripts/experiments/target/fake-frozen-tree")
    judge.write_text(os.path.join(tree, judge.MUTATIONS_TABLE), "# 假的坏实现表\n")
    segment_one = placeholders.SegmentOne(tree="scripts/experiments/target/fake-frozen-tree", snapshot_digest=judge.row_driver.snapshot_digest(tree)[0],
                                          mutations_sha256=judge.sha256_of_file(os.path.join(tree, judge.MUTATIONS_TABLE)), run_date=FAKE_RUN_DATE,
                                          suffix="-r8", dumps="scripts/experiments/target/fake-segment-one-dumps", logs=os.path.join(root, "segment-one-logs"))
    judge.write_text(os.path.join(base.repo, segment_one.timing(placeholders.STEP_1_STEM)),
                     "E7TIMING name=stream_time stream=a elapsed_ns=50000000000\nE7TIMING name=process_wall seconds=75.274 exit=0\n")
    judge.write_text(segment_one.sequence_log, "\n".join(segment_one_lines(segment_one, base.repo, kind)) + "\n")
    revision_8_config = {"tree": segment_one.tree, "H3": {"snapshot_digest": segment_one.snapshot_digest}, "H8": {"mutations_table_sha256": segment_one.mutations_sha256},
                         "H16": {"run_date": FAKE_RUN_DATE, "run_suffix": "-r8", "dumps_root": segment_one.dumps, "logs_directory": segment_one.logs},
                         "H18": placeholders.revision_8.template()["H18"]}
    revision_8_path = judge.write_text(os.path.join(root, "revision-8.json"), json.dumps(revision_8_config, ensure_ascii=False))
    config = placeholders.template()
    config["revision_8_config"] = revision_8_path
    config["H23"] = {"run_date": FAKE_RUN_DATE, "run_suffix": "-r8", "dumps_root": "scripts/experiments/target/e171-r1-dumps/fake-segment-two",
                     "logs_directory": os.path.join(root, "segment-two-logs"), "work_root": os.path.join(root, "segment-two-work")}
    config_path = judge.write_text(os.path.join(root, "revision-10.json"), json.dumps(config, ensure_ascii=False))
    segment_two = placeholders.segment_two_of(config)
    environment = dict(base.environment, SEGMENT_TWO_R10_SELFTEST="1", SEGMENT_TWO_R10_SKIP_CONFIG_CHECK="1", SEGMENT_TWO_R10_YIELD_RECHECK_SECONDS="0.2",
                       SEGMENT_TWO_R10_RESEARCH_ROOT=two.REAL_REPOSITORY, **(extra_environment or {}))
    command = [sys.executable, "-B", os.path.abspath(__file__), mode, "--config", config_path, "--repo", base.repo]
    world = two.WorldTwo(base.root, base.repo, command, environment, segment_two.prefix + "-", segment_two.dumps)
    return WorldTen(world, config_path, segment_one, segment_two)


def product_ten(world: WorldTen, stem: str) -> str:
    return os.path.join(world.base.repo, f"{world.segment_two.prefix}-{stem}{world.segment_two.suffix}.out")


def ran_lines(world: WorldTen) -> list[str]:
    path = os.path.join(world.base.root, "driver.log")
    return [line for line in open(path, encoding="utf-8").read().splitlines() if line.startswith("ran ")] if os.path.exists(path) else []


def whole_table_hosts(world: WorldTen) -> set[str]:
    """整表那一步的行落在哪几台（本机的输出根叫 segment2-rows，别的台那一头叫 rows；跨机对照那一次不算）。"""
    return {one.key_values(line)["host"] for line in ran_lines(world) if " root=segment2-rows " in line + " " or " root=rows " in line + " "}


def timing_of(world: WorldTen) -> placeholders.BaselineTimingOfStepOne:
    return placeholders.resolve_baseline_timing(world.base.repo, world.segment_one)


def cell_overrides_applied(scratch: str) -> tuple[bool, str]:
    world = fake_world_ten(os.path.join(scratch, "world"))
    configure(world.segment_one, world.segment_two)
    names = (one.PEER_KIT_RELATIVE, one.PEER_SNAPSHOT_RELATIVE, one.PEER_OUT_RELATIVE, two.PEER_LAUNCH_RELATIVE)
    attempt = types.SimpleNamespace(attempt_dir="/x/attempt-1", tag="-s2")
    labels = (one.peer_label(attempt, "preflight-h2"), two.peer_label_two(attempt, "rows-row-3-h2"))
    good = (names == world.segment_two.peer_names and all(label.startswith(world.segment_two.label_prefix + "-attempt-1-") for label in labels)
            and one.snapshot_check is snapshot_check_with_the_tree_table and one.allocate is allocate_behind_the_local_gate
            and two.run_queue_two is run_queue_with_stop_file_and_yield)
    return good, f"四个落点 {names}；标签 {labels}"


def cell_mutations_table_checked(scratch: str) -> tuple[bool, str]:
    world = fake_world_ten(os.path.join(scratch, "world"))
    configure(world.segment_one, world.segment_two)
    context = context_of(world.base.repo, world.segment_one, world.segment_two, timing_of(world))
    one.snapshot_check(context)
    judge.write_text(os.path.join(world.base.repo, world.segment_one.tree, judge.MUTATIONS_TABLE), "# 拍完又改了\n")
    try:
        one.snapshot_check(context)
    except one.Stopped as error:
        return "H8" in str(error), f"树里 crates/mutations.tsv 改了：停（{error}）"
    return False, "树里 crates/mutations.tsv 改了却没停"


def cell_segment_one_rows_waiting(scratch: str) -> tuple[bool, str]:
    segment_one = placeholders.SegmentOne("tree", "d" * 64, "e" * 64, FAKE_RUN_DATE, "-r8", "dumps", os.path.join(scratch, "logs"))
    rows_root = os.path.join(scratch, "dumps", "segment1-rows")
    judge.write_text(os.path.join(rows_root, "row-242", "row-result.tsv"), "status\tok\n")
    for attempt, label, record, finished in (("attempt-1", "s12-row-15-h1", {"kind": "local", "step": "s12", "rows": [15]}, False),
                                             ("attempt-2", "s12-row-0-h1", {"kind": "local", "step": "s12", "rows": [0]}, False),
                                             ("attempt-2", "s11-row-269-h2", {"kind": "peer", "step": "s11", "rows": [269]}, True),
                                             ("attempt-2", "s12-carry", {"kind": "carry", "step": "s12", "rows": [23]}, False)):
        judge.write_text(os.path.join(rows_root + "-shards", attempt, label, "shard.json"), json.dumps(record))
        if finished:
            judge.write_text(os.path.join(rows_root + "-shards", attempt, label, "exit"), "0\n")
    started = f"{FAKE_RUN_DATE} {placeholders.ROWS_ORCHESTRATOR_STARTED}（…）"
    judge.write_text(segment_one.sequence_log, started + "\n")
    waiting = segment_one_rows_waiting(scratch, segment_one)
    expected = [f"{step}:{row}" for step in SEGMENT_ONE_ROW_STEPS for row in one.STEP_ROWS[step] if (step, row) not in (("s11", 242), ("s12", 0))]
    judge.write_text(segment_one.sequence_log, f"{started}\n{FAKE_RUN_DATE}   逐行编排退 1\n")
    ended = segment_one_rows_waiting(scratch, segment_one)
    return waiting == expected and ended == [], f"在跑时排着的 {len(waiting)} 行（要 {len(expected)}：第 242 行做完、最近一趟第 0 行在跑）；退了之后 {ended}"


def cell_inputs_checked(scratch: str) -> tuple[bool, str]:
    repo = os.path.join(scratch, "repo")
    files = [judge.write_text(os.path.join(repo, name), f"{name}\n") and name for name in ("a.py", "b.tsv")]
    values = [("tree", "t"), ("baseline_timing_h21", "x-timing.out")]
    first = record_or_check_inputs(repo, "dumps", files, values)
    again = record_or_check_inputs(repo, "dumps", files, values)
    judge.write_text(os.path.join(repo, "b.tsv"), "改了\n")
    try:
        record_or_check_inputs(repo, "dumps", files, values)
    except one.Stopped as error:
        return "记下" in first and "相同" in again, f"第一趟 {first[:20]}；续跑 {again[:20]}；改了一个输入：停（{str(error)[:60]}）"
    return False, "改了一个输入，续跑却没停"


def cell_run_values_checked(scratch: str) -> tuple[bool, str]:
    repo = os.path.join(scratch, "repo")
    files = [judge.write_text(os.path.join(repo, "a.py"), "a\n") and "a.py"]
    record_or_check_inputs(repo, "dumps", files, [("baseline_timing_h21", "x-s2-device-baseline-streams-1-2-r8-timing.out")])
    try:
        record_or_check_inputs(repo, "dumps", files, [("baseline_timing_h21", "x-s2-device-baseline-streams-1-2-attempt2-r8-timing.out")])
    except one.Stopped as error:
        return "baseline_timing_h21" in str(error), f"第 1 步重跑过、T₂ 换了来源：停（{str(error)[:80]}）"
    return False, "T₂ 换了来源，续跑却没停"


def last_line(output: str) -> list[str]:
    return output.strip().splitlines()[-1:]


def cell_e2e_two_hosts(scratch: str) -> tuple[bool, str]:
    world = fake_world_ten(os.path.join(scratch, "world"), extra_environment={"FAKE_DRIVER_TIMEOUT_ROWS": "4"})
    code, output = two.world_run_two(world.base)
    counts, expected = two.ran_counts_two(world.base), two.plain_counts((1, 2, 3, 4, 5, 6), 1, (4,))
    mhr_log = os.path.join(world.base.root, "mhr.log")
    labels = [one.key_values(line)["label"] for line in open(mhr_log, encoding="utf-8")] if os.path.exists(mhr_log) else []
    shards = os.path.join(world.base.repo, world.segment_two.dumps, "segment2-rows-shards")
    fetched = [root for root, _, _ in os.walk(shards) if root.endswith(world.segment_two.peer_names[2])]
    dumps = os.path.join(world.base.repo, world.segment_two.dumps)
    good = (code == 0 and counts == expected and os.path.exists(product_ten(world, "rows-driver"))
            and "cross_host_identical=true" in two.last_verdict(product_ten(world, two.CROSS_HOST_CHECK_STEM))
            and "substitute_collisions=0" in two.last_verdict(product_ten(world, two.COORDINATION_STEM))
            and bool(labels) and all(label.startswith(world.segment_two.label_prefix + "-") for label in labels) and bool(fetched)
            and all(os.path.exists(os.path.join(dumps, name)) for name in (placeholders.INPUTS_FILE, placeholders.RUN_VALUES_FILE)))
    return good, (f"退 {code}；真跑过的次数与预期{'相同' if counts == expected else '不同：' + str(counts)}；别的台标签 {sorted(set(label[:9] for label in labels))}；"
                  f"取回到 H24 落点的分片 {len(fetched)}；末行 {last_line(output)}")


def cell_e2e_stop_file(scratch: str) -> tuple[bool, str]:
    world = fake_world_ten(os.path.join(scratch, "world"))
    judge.write_text(world.segment_one.stop_file, "单元 step-3-o2 红\n")
    code, output = two.world_run_two(world.base)
    return code == 1 and not ran_lines(world) and "停派文件" in output, f"有停派文件：退 {code}，真跑过的行 {len(ran_lines(world))}（要 0）；末行 {last_line(output)}"


def cell_e2e_s17_segment_one(scratch: str) -> tuple[bool, str]:
    """第一段那一处 S17 触发（它的跨机对照产物 verdict 行 cross_host_identical=false）：第二段一行都不起，退 1。"""
    world = fake_world_ten(os.path.join(scratch, "world"))
    judge.write_text(os.path.join(world.base.repo, world.segment_one.product("rows-cross-host-check")),
                     "E7RESULT name=verdict cross_host_identical=false row=0 differing_parts=o3\n")
    code, output = two.world_run_two(world.base)
    return code == 1 and not ran_lines(world) and "S17" in output, f"第一段 S17 触发：退 {code}，真跑过的行 {len(ran_lines(world))}（要 0）；末行 {last_line(output)}"


def cell_e2e_gate_refused(scratch: str) -> tuple[bool, str]:
    world = fake_world_ten(os.path.join(scratch, "world"), kind="gate-not-green")
    code, output = two.world_run_two(world.base)
    return (code == one.EXIT_RUN_CONDITION and not ran_lines(world) and "起跑闸没过" in output,
            f"修订 8 那一趟第 2 步没绿：退 {code}（要 78），真跑过的行 {len(ran_lines(world))}；末行 {last_line(output)}")


def cell_e2e_local_gate_closed(scratch: str) -> tuple[bool, str]:
    """本机闸一直没开：整表的行只落在第二台；跨机对照要本机，此刻放不下，退 78（第 7 步绿了再起就是续跑）。"""
    world = fake_world_ten(os.path.join(scratch, "world"), kind="before-step-7", rows="1,2,3")
    code, output = two.world_run_two(world.base)
    hosts = whole_table_hosts(world)
    return (code == one.EXIT_RUN_CONDITION and hosts == {"2"} and "跨机对照" in output,
            f"退 {code}（要 78）；整表的行落在 {sorted(hosts)}（要只有第二台）；末行 {last_line(output)}")


def wait_for(predicate, seconds: float) -> bool:
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        if predicate():
            return True
        time.sleep(0.05)
    return predicate()


def cell_e2e_local_gate_opens(scratch: str) -> tuple[bool, str]:
    """起跑时本机闸没开（第二台一槽）；跑起来之后修订 8 日志里第 7 步绿、逐行编排起过，编排器的加槽把本机加进来。"""
    resources = [one.resources_entry(1, 10, 25, 25), one.resources_entry(2, 5, 25, 25)]
    world = fake_world_ten(os.path.join(scratch, "world"), kind="before-step-7", resources=resources,
                           extra_environment={"FAKE_DRIVER_ROW_SECONDS": "0.3", "ROWS_ORCHESTRATOR_TWO_REGROW_SECONDS": "0.2"})
    process = subprocess.Popen(world.base.command, cwd=world.base.repo, env=world.base.environment, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    wait_for(lambda: bool(ran_lines(world)), 60)
    with open(world.segment_one.sequence_log, "a", encoding="utf-8") as handle:
        handle.write("\n".join(segment_one_lines(world.segment_one, world.base.repo, "finished")[-3:]) + "\n")
    output, _ = process.communicate(timeout=240)
    hosts = whole_table_hosts(world)
    good = process.returncode == 0 and hosts == {"1", "2"} and "加槽：本机" in output
    return good, f"退 {process.returncode}；整表的行落在 {sorted(hosts)}；加槽那一行 {'有' if '加槽：本机' in output else '没有'}；末行 {last_line(output)}"


def cell_e2e_yield(scratch: str) -> tuple[bool, str]:
    """修订 8 的逐行编排在跑、它的行都还排着：第二段一行都不起；那些行做完之后接着派，跑完退 0。"""
    world = fake_world_ten(os.path.join(scratch, "world"), kind="rows-running", rows="1,2")
    process = subprocess.Popen(world.base.command, cwd=world.base.repo, env=world.base.environment, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    sequence = os.path.join(world.segment_two.logs, "sequence.log")
    wait_for(lambda: bool(ran_lines(world)) or (os.path.exists(sequence) and "让路：第一段逐行还有" in open(sequence, encoding="utf-8").read()), 60)
    time.sleep(1.0)
    entered = os.path.exists(sequence) and "让路：第一段逐行还有" in open(sequence, encoding="utf-8").read()
    during = len(ran_lines(world))
    rows_root = os.path.join(world.base.repo, world.segment_one.dumps, "segment1-rows")
    for step in SEGMENT_ONE_ROW_STEPS:
        for row in one.STEP_ROWS[step]:
            judge.write_text(os.path.join(rows_root + one.STEPS[step].root_suffix, f"row-{row}", "row-result.tsv"), "status\tok\n")
    output, _ = process.communicate(timeout=240)
    good = entered and during == 0 and process.returncode == 0 and "让路结束" in output
    return good, f"让路那一行 {'有' if entered else '没有'}；让路时真跑过的行 {during}（要 0）；第一段的行做完之后退 {process.returncode}；末行 {last_line(output)}"


def cell_plan_writes_nothing(scratch: str) -> tuple[bool, str]:
    world = fake_world_ten(os.path.join(scratch, "world"), mode="plan")
    code, output = two.world_run_two(world.base)
    row_lines = [line for line in output.splitlines() if line.startswith("行 ") and "--no-executor" in line]
    results = os.path.join(world.base.repo, "research/results")
    written = [name for name in os.listdir(results) if name.startswith(os.path.basename(world.segment_two.prefix))]
    created = [path for path in (os.path.join(world.base.repo, world.segment_two.dumps), world.segment_two.logs) if os.path.exists(path)]
    good = code == 0 and len(row_lines) == 6 and "依赖·起跑闸" in output and "别的台那一行" in output and not written and not created and not ran_lines(world)
    return good, f"退 {code}；行命令 {len(row_lines)} 条（要 6）；写了的产物 {written}、建了的目录 {created}（都要空）"


def segment_two_selftest_cell(scratch: str) -> tuple[bool, str]:
    environment = {key: value for key, value in os.environ.items() if not key.startswith(("SEGMENT_TWO_R10_", "ROWS_ORCHESTRATOR"))}
    completed = subprocess.run([sys.executable, "-B", SEGMENT_TWO_ORCHESTRATOR_PATH, "--selftest"], capture_output=True, text=True, env=environment, timeout=900)
    got = (completed.stdout.strip().splitlines() or [""])[-1]
    return completed.returncode == 0 and got == SEGMENT_TWO_SELFTEST_LAST_LINE, f"rows_orchestrator_segment_two.py --selftest 末行 {got}（要 {SEGMENT_TWO_SELFTEST_LAST_LINE}）"


SELFTEST_CELLS = {
    "overrides-applied": cell_overrides_applied, "mutations-table-checked": cell_mutations_table_checked,
    "segment-one-rows-waiting": cell_segment_one_rows_waiting, "inputs-checked": cell_inputs_checked, "run-values-checked": cell_run_values_checked,
    "e2e-two-hosts": cell_e2e_two_hosts, "e2e-stop-file": cell_e2e_stop_file, "e2e-s17-segment-one": cell_e2e_s17_segment_one,
    "e2e-gate-refused": cell_e2e_gate_refused,
    "e2e-local-gate-closed": cell_e2e_local_gate_closed, "e2e-local-gate-opens": cell_e2e_local_gate_opens, "e2e-yield": cell_e2e_yield,
    "plan-writes-nothing": cell_plan_writes_nothing, "segment-two-orchestrator-selftest": segment_two_selftest_cell,
}
SELFTEST_SWITCHES = {"peer-names-unchanged": "overrides-applied", "mutations-table-skipped": "mutations-table-checked",
                     "local-gate-ignored": "e2e-local-gate-closed", "running-shards-ignored": "segment-one-rows-waiting", "yield-ignored": "e2e-yield",
                     "stop-file-ignored": "e2e-stop-file", "s17-ignored": "e2e-s17-segment-one", "gate-ignored": "e2e-gate-refused", "inputs-unchecked": "inputs-checked",
                     "run-values-unchecked": "run-values-checked", "plan-writes-inputs": "plan-writes-nothing"}


def selftest(argument_list: list[str]) -> int:
    os.environ[SELFTEST_VARIABLE] = "1"
    return harness.run_selftest(os.path.abspath(__file__), argument_list, SELFTEST_CELLS, SELFTEST_SWITCHES, BREAK_VARIABLE, SELFTEST_VARIABLE)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
