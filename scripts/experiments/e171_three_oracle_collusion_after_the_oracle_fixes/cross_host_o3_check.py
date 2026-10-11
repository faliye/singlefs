#!/usr/bin/env python3
"""E171 第 1 次重跑第一段·修订 6 树 `-resume2` 那一趟：跨机一致对照（停机条款 S17，登记「主 agent 认定：多机跑法交回」第 4 条）的单独入口。
在已经跑完的两份行目录（第 12 步 row-0 本机那一次、跨机对照 X 第二台那一次）上重出对照产物；一行都不跑、不起驱动、不起 cargo、不碰别的台。

与编排器 rows_orchestrator.py（sha256 7bfbdb84…，不改）的关系：row 行、device.out 的 E7RESULT 行、导出 z1 五个文件三样的比法与产物的行形
照抄自它——前三样直接调它的函数（row_line_without_timing、device_result_lines、dump_differences、row_line、read_ledger、host_of），开跑先核它的
sha256，不是 7bfbdb84… 就退 3。只改 o3 那一样：在它的去法（normalized_o3_lines：去 name=config、name=state_time 两种行与键名以 _ns 结尾的字段，
原样调它）之后，再去三个只随进程号、工作进程数与调度变的键，并按整行排序再比：
  name=state 行的 worker=（冻结执行器 executor_main.py 判那一态的工作进程号 os.getpid()）；
  name=timing 行的 workers=（--workers 参数，即每槽线程）与 worker_processes=（这一批实际判过态的工作进程个数）；
  行序（多进程时 pool.imap_unordered 按完成先后写 name=state 行；每行自带 stream=、state=、batch=，次序不带判定）。
三个键只在点名的那一种行里去：别的行里出现同名的键照比（收严：编排器的去法里没有这三个键，这里也不把它们放宽到别的行）。
判定字段（digest、strict、provisional、gaps、landed、refusal、allowed、chosen_root……与各汇总行的计数）一个不去。排序之后比的是多重集：
多一行、少一行、同一行出现次数不同都判不同。

用法：
  cross_host_o3_check.py compare --repo R --step-12-row-0 <行目录> --cross-host-row-0 <行目录> --step-12-product <产物> --cross-host-product <产物>
                                 --ledger <row-hosts.tsv>
      把对照产物写到标准输出（落盘由调用的一方用排他的重定向写新文件名）；末行是 name=verdict。
  cross_host_o3_check.py --selftest
      假行目录走一遍各格，末尾逐个打开弄坏开关核它红在点名格；末行「自证 N 项，红 M」。
退出码：0 四样都同；1 有一样不同（停机条款 S17 的「不同」）；2 用法错，或平常跑时设了弄坏开关；3 编排器不是 7bfbdb84… 那一份。

弄坏开关 CROSS_HOST_O3_BREAK=<项>（只在 --selftest 起的子进程里认，CROSS_HOST_O3_SELFTEST=1；别处设了退 2）。项名、弄坏哪一处、打开后红在哪一格：
  o3-keeps-worker                      name=state 行的 worker= 不去              o3-same-after-process-identifiers-changed
  o3-keeps-workers                     name=timing 行的 workers=、worker_processes= 不去  o3-same-after-thread-count-changed
  o3-keeps-line-order                  不排序，逐行按原次序比                    o3-same-after-line-order-shuffled
  o3-drops-strict                      连 strict= 也去掉                         o3-differs-after-strict-changed
  o3-compares-sets                     比去重之后的行集合，不比多重集            o3-differs-after-one-line-duplicated
  o3-drops-worker-keys-in-every-kind   三个键在每一种行里都去                    o3-differs-when-worker-key-appears-in-another-line-kind
  parts-ignore-dumps                   不比导出那一样                            parts-dump-difference-detected
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import os
import random
import shutil
import subprocess
import sys
import tempfile
import types

# 编排器取同一目录里入库的那一份（与原先草稿 e171-r1-segment1-rev6/ 里那份逐字节相同，开跑照旧核 sha256）
ORCHESTRATOR_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "rows_orchestrator.py")
ORCHESTRATOR_SHA256 = "7bfbdb84559476033853794499836f752fdf8a3e803e17cd9539ae49ee3b7e0e"
# 去法：键只在点名的那一种行里去（行种类 → 键）
SCHEDULING_KEYS_BY_LINE_KIND = {"name=state": ("worker",), "name=timing": ("workers", "worker_processes")}
EVERY_SCHEDULING_KEY = ("worker", "workers", "worker_processes")
O3_DROPPED_TEXT = "name=config,name=state_time,*_ns,name=state:worker,name=timing:workers,name=timing:worker_processes,line_order"
BREAK_SWITCHES = {
    "o3-keeps-worker": "o3-same-after-process-identifiers-changed",
    "o3-keeps-workers": "o3-same-after-thread-count-changed",
    "o3-keeps-line-order": "o3-same-after-line-order-shuffled",
    "o3-drops-strict": "o3-differs-after-strict-changed",
    "o3-compares-sets": "o3-differs-after-one-line-duplicated",
    "o3-drops-worker-keys-in-every-kind": "o3-differs-when-worker-key-appears-in-another-line-kind",
    "parts-ignore-dumps": "parts-dump-difference-detected",
}
EXIT_SAME = 0
EXIT_DIFFERENT = 1
EXIT_USAGE = 2
EXIT_WRONG_ORCHESTRATOR = 3


class WrongOrchestrator(Exception):
    """编排器文件的 sha256 不是 7bfbdb84… 那一份：照抄的三样比法不再是跑那一趟用的那一份。"""


def broken(item: str) -> bool:
    return item in os.environ.get("CROSS_HOST_O3_BREAK", "").split(",")


def sha256_of_file(path: str) -> str:
    with open(path, "rb") as handle:
        return hashlib.sha256(handle.read()).hexdigest()


def load_orchestrator(path: str = ORCHESTRATOR_PATH) -> types.ModuleType:
    """按路径载入编排器（不改它、不写 .pyc）；sha256 不对就拒。"""
    actual = sha256_of_file(path)
    if actual != ORCHESTRATOR_SHA256:
        raise WrongOrchestrator(f"{path} 的 sha256 是 {actual}，不是 {ORCHESTRATOR_SHA256}")
    sys.dont_write_bytecode = True
    specification = importlib.util.spec_from_file_location("rows_orchestrator_7bfbdb84", path)
    module = importlib.util.module_from_spec(specification)
    sys.modules[specification.name] = module  # dataclasses 在类定义时按 __module__ 去 sys.modules 里找这个模块
    specification.loader.exec_module(module)
    return module


def line_kind(line: str) -> str:
    return line.split(" ", 1)[0]


def without_scheduling_keys(line: str) -> str:
    if broken("o3-drops-worker-keys-in-every-kind"):
        dropped = EVERY_SCHEDULING_KEY
    else:
        dropped = SCHEDULING_KEYS_BY_LINE_KIND.get(line_kind(line), ())
        if broken("o3-keeps-worker"):
            dropped = tuple(key for key in dropped if key != "worker")
        if broken("o3-keeps-workers"):
            dropped = tuple(key for key in dropped if key not in ("workers", "worker_processes"))
    if broken("o3-drops-strict"):
        dropped = dropped + ("strict",)
    return " ".join(token for token in line.split(" ") if token.split("=", 1)[0] not in dropped)


def o3_lines_without_scheduling(orchestrator: types.ModuleType, row_directory: str) -> list[str]:
    """编排器的去法原样调一次，再去三个调度键、按整行排序。o3.out 不在时交回空表（对照那一格因此判不同，同编排器）。"""
    kept = [without_scheduling_keys(line) for line in orchestrator.normalized_o3_lines(row_directory)]
    if broken("o3-compares-sets"):
        kept = list(dict.fromkeys(kept))
    return kept if broken("o3-keeps-line-order") else sorted(kept)


def cross_host_parts(orchestrator: types.ModuleType, line_a: str, directory_a: str, line_b: str, directory_b: str) -> list[tuple[str, bool, str]]:
    """四样各一行：(样, 相同, 细节)。前三样同编排器 cross_host_parts（调它的函数），o3 那一样换成去调度键、排序之后比。"""
    parts = [("row_line", orchestrator.row_line_without_timing(line_a) == orchestrator.row_line_without_timing(line_b),
              "dropped=" + ",".join(orchestrator.TIMING_COLUMNS))]
    device_a = [line for line in orchestrator.device_result_lines(directory_a) if line.startswith("E7RESULT ")]
    device_b = [line for line in orchestrator.device_result_lines(directory_b) if line.startswith("E7RESULT ")]
    parts.append(("device_results", bool(device_a) and device_a == device_b, f"lines={len(device_a)}/{len(device_b)}"))
    compared, differing = orchestrator.dump_differences(directory_a, directory_b)
    if broken("parts-ignore-dumps"):
        compared, differing = max(compared, 1), []
    parts.append(("dump_files", compared > 0 and not differing, f"compared={compared} differing={','.join(differing) or 'none'}"))
    o3_a = o3_lines_without_scheduling(orchestrator, directory_a)
    o3_b = o3_lines_without_scheduling(orchestrator, directory_b)
    parts.append(("o3", bool(o3_a) and o3_a == o3_b, f"lines={len(o3_a)}/{len(o3_b)} dropped={O3_DROPPED_TEXT}"))
    return parts


def cross_host_product(orchestrator: types.ModuleType, parts: list[tuple[str, bool, str]], host_a: int, host_b: int,
                       inputs_line: str) -> tuple[str, bool]:
    """交回 (产物原文, 是否停机)。行形同编排器 cross_host_product，o3_dropped 写新的去法，另加一行 name=cross_host_inputs 记读了哪几份。"""
    lines = ["E7RESULT name=cross_host_columns row=0 row_line_dropped=" + ",".join(orchestrator.TIMING_COLUMNS)
             + " device=E7RESULT_lines dump=" + ",".join(orchestrator.Z1_FILES) + f" o3_dropped={O3_DROPPED_TEXT}", inputs_line]
    for part, identical, detail in parts:
        lines.append(f"E7RESULT name=cross_host_part part={part} identical={'true' if identical else 'false'} {detail}")
    differing = [part for part, identical, _ in parts if not identical]
    lines.append(f"E7RESULT name=verdict cross_host_identical={'false' if differing else 'true'} row=0 host_of_step_12_row_0={host_a} "
                 f"host_of_cross_host_row_0={host_b} parts={len(parts)} differing_parts={','.join(differing) or 'none'}")
    return "\n".join(lines) + "\n", bool(differing)


def compare(arguments: argparse.Namespace) -> int:
    orchestrator = load_orchestrator()
    if os.path.basename(arguments.ledger) != "row-hosts.tsv":
        print(f"--ledger 要指编排器的账 row-hosts.tsv，给的是 {arguments.ledger}", file=sys.stderr)
        return EXIT_USAGE
    ledger = orchestrator.read_ledger(types.SimpleNamespace(shards=os.path.dirname(os.path.abspath(arguments.ledger))))
    holder = types.SimpleNamespace()
    host_a, host_b = orchestrator.host_of(holder, ledger, "s12", 0), orchestrator.host_of(holder, ledger, "sx", 0)
    if host_a == host_b:
        print(f"跨机对照的两次都在第 {host_a} 台（账 {arguments.ledger}）：对照没做成", file=sys.stderr)
        return EXIT_DIFFERENT
    line_a = orchestrator.row_line(arguments.step_12_product, 0)
    line_b = orchestrator.row_line(arguments.cross_host_product, 0)
    parts = cross_host_parts(orchestrator, line_a, arguments.step_12_row_0, line_b, arguments.cross_host_row_0)

    def relative(path: str) -> str:
        return os.path.relpath(os.path.abspath(path), os.path.abspath(arguments.repo))

    o3_hashes = [sha256_of_file(os.path.join(directory, "o3.out")) if os.path.exists(os.path.join(directory, "o3.out")) else "missing"
                 for directory in (arguments.step_12_row_0, arguments.cross_host_row_0)]
    inputs_line = (f"E7RESULT name=cross_host_inputs entry_sha256={sha256_of_file(os.path.abspath(__file__))} "
                   f"orchestrator_sha256={ORCHESTRATOR_SHA256} step_12_row_0={relative(arguments.step_12_row_0)} "
                   f"cross_host_row_0={relative(arguments.cross_host_row_0)} step_12_product={relative(arguments.step_12_product)} "
                   f"cross_host_product={relative(arguments.cross_host_product)} ledger={relative(arguments.ledger)} "
                   f"o3_sha256={o3_hashes[0]},{o3_hashes[1]}")
    text, stop = cross_host_product(orchestrator, parts, host_a, host_b, inputs_line)
    sys.stdout.write(text)
    return EXIT_DIFFERENT if stop else EXIT_SAME


def main(argument_list: list[str]) -> int:
    if os.environ.get("CROSS_HOST_O3_BREAK") and os.environ.get("CROSS_HOST_O3_SELFTEST") != "1":
        print("✗ CROSS_HOST_O3_BREAK 只在 --selftest 起的子进程里认；平常跑时设了它就拒（退 2）", file=sys.stderr)
        return EXIT_USAGE
    try:
        if argument_list[:1] == ["--selftest"]:
            return selftest(argument_list[1:])
        parser = argparse.ArgumentParser(description="E171 第一段 -resume2 的跨机一致对照：在已跑完的两份行目录上重出对照产物")
        parser.add_argument("mode", choices=("compare",))
        for name in ("--repo", "--step-12-row-0", "--cross-host-row-0", "--step-12-product", "--cross-host-product", "--ledger"):
            parser.add_argument(name, required=True)
        arguments = parser.parse_args(argument_list)
        return compare(arguments)
    except WrongOrchestrator as error:
        print(f"✗ {error}", file=sys.stderr)
        return EXIT_WRONG_ORCHESTRATOR


# ── 自证：假的 o3.out（同冻结执行器 executor_main.py 的行形），各格两边各造一份 ──
FAKE_STREAMS = ("new_pool_file_creation", "fixed_script_through_e_then_unmount")
FAKE_STATES_PER_BATCH = 8
FAKE_PROCESS_COUNT = 3


def fake_state_line(stream: str, state: int, batch: int, worker_identity: int, elapsed: int) -> str:
    strict = "in" if state % 4 else "not_in"
    return (f"name=state stream={stream} state={state} layer0={state + 40} segment={state % 3} batch={batch} digest=match strict={strict} "
            f"provisional={strict} gaps=- landed=(1,{state % 4}) refusal=- allowed=[(1,2),(1,3)] m=3 ack=2 begun=3 chosen_root=(1,3) "
            f"chosen_below_largest=0 chosen_below_largest_self_verified=0 slot_one=1 applied=1 last_counter=3 stop=end path=a "
            f"anchor_unreadable=0 torn=0 strict_before_witness={strict} provisional_before_witness={strict} witness_counter=3 "
            f"witness_branch=last_record witness_compared=3 witness_true=false ack_rotation_tail=2 tail_check=true failed_predicates=- "
            f"time_ns={elapsed} worker={worker_identity}")


def fake_o3_lines(process_identifier_base: int, worker_count: int, order_seed: int) -> list[str]:
    """进程号 = 基数 + 态号 % 3（与次序无关），workers= 与 worker_processes= 取 worker_count，order_seed 非 0 时每批 name=state 行打乱。"""
    lines = [f"name=config dumps=/fake/{process_identifier_base} streams={','.join(FAKE_STREAMS)} batches=1,2 workers={worker_count} python=3.12.3"]
    lines += [f"name=load stream={stream} load_ns={process_identifier_base * 7} prepare_ns=11 ops=40 writes=12 calls=3 sampled_states=16 torn_writes=0"
              for stream in FAKE_STREAMS]
    for batch in (1, 2):
        states = [(stream, number) for stream in FAKE_STREAMS for number in range(batch * 100, batch * 100 + FAKE_STATES_PER_BATCH // 2)]
        if order_seed:
            random.Random(order_seed * 10 + batch).shuffle(states)
        lines += [fake_state_line(stream, number, batch, process_identifier_base + number % FAKE_PROCESS_COUNT, 1000 + process_identifier_base + number)
                  for stream, number in states]
        lines.append(f"name=timing batch={batch} workers={worker_count} states={len(states)} total_ns={9000 + worker_count} sum_state_ns=8000 "
                     f"largest_worker_sum_ns={4000 // worker_count} worker_processes={worker_count} contains_self_check=pass")
        lines += [f"name=state_time batch={batch} workers={worker_count} stream={stream} states=4 median_ns={worker_count} p95_ns=1 max_ns=2 mean_ns=1"
                  for stream in FAKE_STREAMS]
        lines += [f"name=verdict batch={batch} stream={stream} states=4 strict_spec_gap_states=0 strict_spec_gap_identifiers=zero strict_given_states=4 "
                  f"provisional_in=3 provisional_not_in=1 judgement_failures=0 digest_mismatches=0" for stream in FAKE_STREAMS]
    lines += ["name=freeze file=executor_main.py sha256=00", "name=freeze_seed sha256_prefix16=ce39b484ec109c7a", "name=complete"]
    return lines


def replace_in_first(lines: list[str], kind: str, old: str, new: str) -> list[str]:
    """把第一条 kind 行（且含 old）里的 old 换成 new；找不到就抛错（那一格的编辑没生效，不许当成绿）。"""
    for index, line in enumerate(lines):
        if line_kind(line) == kind and old in line:
            return lines[:index] + [line.replace(old, new, 1)] + lines[index + 1:]
    raise AssertionError(f"假 o3.out 里没有含 {old!r} 的 {kind} 行")


def first_state_index(lines: list[str]) -> int:
    return next(index for index, line in enumerate(lines) if line_kind(line) == "name=state")


O3_EDITS = {
    "o3-differs-after-strict-changed": lambda lines: replace_in_first(lines, "name=state", " strict=in ", " strict=not_in "),
    "o3-differs-after-provisional-changed": lambda lines: replace_in_first(lines, "name=state", " provisional=in ", " provisional=not_in "),
    "o3-differs-after-chosen-root-changed": lambda lines: replace_in_first(lines, "name=state", " chosen_root=(1,3) ", " chosen_root=(1,2) "),
    "o3-differs-after-digest-changed": lambda lines: replace_in_first(lines, "name=state", " digest=match ", " digest=mismatch "),
    "o3-differs-after-allowed-changed": lambda lines: replace_in_first(lines, "name=state", " allowed=[(1,2),(1,3)] ", " allowed=[(1,3)] "),
    "o3-differs-after-verdict-count-changed": lambda lines: replace_in_first(lines, "name=verdict", " provisional_not_in=1 ", " provisional_not_in=2 "),
    "o3-differs-after-timing-state-count-changed": lambda lines: replace_in_first(lines, "name=timing", " states=8 ", " states=7 "),
    "o3-differs-after-one-state-line-added": lambda lines: lines[:-1] + [fake_state_line(FAKE_STREAMS[0], 999, 2, 1, 1)] + lines[-1:],
    "o3-differs-after-one-state-line-removed": lambda lines: lines[:first_state_index(lines)] + lines[first_state_index(lines) + 1:],
    "o3-differs-after-one-line-duplicated": lambda lines: lines[:first_state_index(lines) + 1] + lines[first_state_index(lines):],
    "o3-differs-when-worker-key-appears-in-another-line-kind": lambda lines: replace_in_first(
        lines, "name=verdict", " judgement_failures=0 ", " judgement_failures=0 worker=12 "),
}


def write_o3(directory: str, lines: list[str]) -> str:
    os.makedirs(directory, exist_ok=True)
    with open(os.path.join(directory, "o3.out"), "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")
    return directory


def o3_same(orchestrator: types.ModuleType, scratch: str, name: str, lines_a: list[str], lines_b: list[str]) -> bool:
    first = o3_lines_without_scheduling(orchestrator, write_o3(os.path.join(scratch, name, "a"), lines_a))
    second = o3_lines_without_scheduling(orchestrator, write_o3(os.path.join(scratch, name, "b"), lines_b))
    return bool(first) and first == second


def cells_o3(orchestrator: types.ModuleType, scratch: str) -> list[tuple[str, bool, str]]:
    base = fake_o3_lines(100, 6, 0)
    results = []
    same_pairs = {
        "o3-same-after-process-identifiers-changed": fake_o3_lines(700, 6, 0),
        "o3-same-after-thread-count-changed": fake_o3_lines(100, 10, 0),
        "o3-same-after-line-order-shuffled": fake_o3_lines(100, 6, 7),
        "o3-same-after-all-three-changed": fake_o3_lines(700, 10, 7),
    }
    for name, other in same_pairs.items():
        results.append((name, o3_same(orchestrator, scratch, name, base, other), "两边只差进程号、线程数、行序之一（或三样都差），要判同"))
    # 这几格的输入真分得出：原样逐字不同，编排器 7bfbdb84… 的去法判不同，打乱的那一份 name=state 行次序真变了
    other = same_pairs["o3-same-after-all-three-changed"]
    old_a = orchestrator.normalized_o3_lines(write_o3(os.path.join(scratch, "old", "a"), base))
    old_b = orchestrator.normalized_o3_lines(write_o3(os.path.join(scratch, "old", "b"), other))
    results.append(("o3-old-normalization-differs-on-the-scheduling-only-pair", base != other and old_a != old_b,
                    "同一对输入在编排器原去法下判不同（复现 -resume2 那一次 o3=不同）"))
    shuffled = [line for line in same_pairs["o3-same-after-line-order-shuffled"] if line_kind(line) == "name=state"]
    results.append(("o3-shuffle-really-reorders-state-lines", shuffled != [line for line in base if line_kind(line) == "name=state"],
                    "打乱那一份的 name=state 行次序与基准不同"))
    for name, edit in O3_EDITS.items():
        edited = edit(fake_o3_lines(700, 10, 7))
        results.append((name, not o3_same(orchestrator, scratch, name, base, edited), "另一边带调度差之外再改这一处，要判不同"))
    empty_a, empty_b = os.path.join(scratch, "empty", "a"), os.path.join(scratch, "empty", "b")
    os.makedirs(empty_a)
    os.makedirs(empty_b)
    results.append(("o3-missing-on-both-sides-is-not-same",
                    not (bool(o3_lines_without_scheduling(orchestrator, empty_a)) and o3_lines_without_scheduling(orchestrator, empty_a)
                         == o3_lines_without_scheduling(orchestrator, empty_b)), "两边都没有 o3.out 不算同（同编排器）"))
    return results


def cells_parts_and_product(orchestrator: types.ModuleType, scratch: str) -> list[tuple[str, bool, str]]:
    results = []
    base_line = orchestrator.write_fake_row(os.path.join(scratch, "parts-a"), workers=5)
    outcomes = {}
    for salt in ("", "row", "device", "dump", "o3"):
        directory = os.path.join(scratch, f"parts-b-{salt or 'same'}")
        other_line = orchestrator.write_fake_row(directory, salt, workers=7)
        parts = cross_host_parts(orchestrator, base_line, os.path.join(scratch, "parts-a"), other_line, directory)
        outcomes[salt or "same"] = [name for name, same, _ in parts if not same]
    for salt, expected in (("same", []), ("row", ["row_line"]), ("device", ["device_results"]), ("dump", ["dump_files"]), ("o3", ["o3"])):
        name = "parts-all-same" if salt == "same" else f"parts-{salt}-difference-detected"
        results.append((name, outcomes[salt] == expected, f"掺 {salt} 时不同的那几样：{outcomes[salt]}"))
    same_parts = [("row_line", True, "a"), ("device_results", True, "b"), ("dump_files", True, "c"), ("o3", True, "d")]
    text, stop = cross_host_product(orchestrator, same_parts, 1, 2, "E7RESULT name=cross_host_inputs fake=1")
    results.append(("product-all-same-verdict-true", not stop and "cross_host_identical=true" in text and "differing_parts=none" in text, "四样都同"))
    text, stop = cross_host_product(orchestrator, same_parts[:3] + [("o3", False, "d")], 1, 2, "E7RESULT name=cross_host_inputs fake=1")
    results.append(("product-o3-different-stops", stop and "cross_host_identical=false" in text and "differing_parts=o3" in text, "o3 不同要停机"))
    results.append(("product-names-the-new-drops", text.splitlines()[0].endswith(" o3_dropped=" + O3_DROPPED_TEXT)
                    and O3_DROPPED_TEXT.startswith("name=config,name=state_time,*_ns,"), "首行 o3_dropped 写全旧去法加新去的三键与行序"))
    altered = os.path.join(scratch, "rows_orchestrator_altered.py")
    with open(ORCHESTRATOR_PATH, "rb") as source, open(altered, "wb") as target:
        target.write(source.read() + b"\n")
    try:
        load_orchestrator(altered)
        refused = False
    except WrongOrchestrator:
        refused = True
    results.append(("orchestrator-hash-checked", refused and orchestrator.__name__ == "rows_orchestrator_7bfbdb84", "编排器多一个字节就拒"))
    return results


def cells_end_to_end(scratch: str) -> list[tuple[str, bool, str]]:
    """compare 用法当子进程起：两份行目录、两份产物、账；四样都同退 0，改一个判定字段退 1，两次在同一台退 1 且不出产物，平常跑时设弄坏开关退 2。"""
    orchestrator = load_orchestrator()
    results = []
    for case, edit, ledger_text, expected_exit in (
            ("same", None, "sx\t0\t2\tattempt-1\tfetched\ns12\t0\t1\tattempt-1\tlocal\n", EXIT_SAME),
            ("strict", O3_EDITS["o3-differs-after-strict-changed"], "sx\t0\t2\tattempt-1\tfetched\n", EXIT_DIFFERENT),
            ("same-host", None, "sx\t0\t1\tattempt-1\tlocal\n", EXIT_DIFFERENT)):
        root = os.path.join(scratch, f"e2e-{case}")
        line_a = orchestrator.write_fake_row(os.path.join(root, "rows", "row-0"), workers=6)
        line_b = orchestrator.write_fake_row(os.path.join(root, "rows-cross-host", "row-0"), workers=10)
        write_o3(os.path.join(root, "rows", "row-0"), fake_o3_lines(100, 6, 0))
        lines_b = fake_o3_lines(700, 10, 7)
        write_o3(os.path.join(root, "rows-cross-host", "row-0"), edit(lines_b) if edit else lines_b)
        os.makedirs(os.path.join(root, "rows-shards"))
        for file_name, text in (("step-12.out", line_a + "\n"), ("cross-host.out", line_b + "\n"), ("rows-shards/row-hosts.tsv", ledger_text)):
            with open(os.path.join(root, file_name), "w", encoding="utf-8") as handle:
                handle.write(text)
        command = [sys.executable, "-B", os.path.abspath(__file__), "compare", "--repo", root, "--step-12-row-0", os.path.join(root, "rows", "row-0"),
                   "--cross-host-row-0", os.path.join(root, "rows-cross-host", "row-0"), "--step-12-product", os.path.join(root, "step-12.out"),
                   "--cross-host-product", os.path.join(root, "cross-host.out"), "--ledger", os.path.join(root, "rows-shards", "row-hosts.tsv")]
        environment = {key: value for key, value in os.environ.items() if key not in ("CROSS_HOST_O3_BREAK", "CROSS_HOST_O3_SELFTEST")}
        completed = subprocess.run(command, capture_output=True, text=True, env=environment)
        verdicts = [line for line in completed.stdout.splitlines() if line.startswith("E7RESULT name=verdict ")]
        product_ok = {"same": verdicts[-1:] and "cross_host_identical=true" in verdicts[-1],
                      "strict": verdicts[-1:] and "cross_host_identical=false" in verdicts[-1] and "differing_parts=o3" in verdicts[-1],
                      "same-host": completed.stdout == ""}[case]
        results.append((f"compare-end-to-end-{case}", completed.returncode == expected_exit and bool(product_ok),
                        f"退 {completed.returncode}（要 {expected_exit}），判决行 {verdicts[-1:]}"))
        if case == "same":
            refused = subprocess.run(command, capture_output=True, text=True, env=dict(environment, CROSS_HOST_O3_BREAK="o3-keeps-worker"))
            results.append(("break-switch-refused-outside-selftest", refused.returncode == EXIT_USAGE and refused.stdout == "",
                            f"平常跑时设弄坏开关退 {refused.returncode}（要 2）"))
    return results


def selftest(argument_list: list[str]) -> int:
    """各格跑一遍；不带 --cells-only 时再逐个打开弄坏开关（子进程），核它红在点名格。"""
    cells_only = argument_list[:1] == ["--cells-only"]
    orchestrator = load_orchestrator()
    scratch = tempfile.mkdtemp(prefix="e171-cross-host-o3-selftest-")
    try:
        results = cells_o3(orchestrator, scratch) + cells_parts_and_product(orchestrator, scratch)
        if not cells_only:
            results += cells_end_to_end(scratch)
    finally:
        shutil.rmtree(scratch)
    for name, passed, detail in results:
        print(f"{'✓' if passed else '✗'} {name}：{detail}")
    if cells_only:
        return 0 if all(passed for _, passed, _ in results) else 1
    for switch, named_cell in BREAK_SWITCHES.items():
        completed = subprocess.run([sys.executable, "-B", os.path.abspath(__file__), "--selftest", "--cells-only"], capture_output=True, text=True,
                                   env=dict(os.environ, CROSS_HOST_O3_SELFTEST="1", CROSS_HOST_O3_BREAK=switch))
        red_cells = [line[2:].split("：", 1)[0] for line in completed.stdout.splitlines() if line.startswith("✗ ")]
        passed = completed.returncode == 1 and named_cell in red_cells
        results.append((f"break-{switch}", passed, f"红的格 {red_cells}"))
        print(f"{'✓' if passed else '✗'} 弄坏开关 {switch} 红在点名格 {named_cell}：子进程退 {completed.returncode}，红的格 {red_cells}")
    red = sum(not passed for _, passed, _ in results)
    print(f"自证 {len(results)} 项，红 {red}")
    return 0 if red == 0 else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
