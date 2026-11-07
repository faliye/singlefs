#!/usr/bin/env python3
# admission: always 量的是此刻这台机器连此刻那份 prune-server 的读写快慢，上一次量的数不替这一次作保
# run-condition: command python3
"""照 Python 三家一趟的读写形态对着一份临时 prune-server 各跑一趟，报每家每秒多少条、失败几条；不对真服务跑。
形态照判定库存法事实表第 2 节（records/2026-11-01-判定库服务化.md 批次 1 交回的调查员报告）：
  verdict-library       每条一次 lookup、判完一次 record（形式化、e165 在判法进程里逐条；lkmm、QEMU 经命令行，进程开销不在这里量）
  unit-verdicts         一格一个进程：进程里逐条 lookup，判完逐条 record（admission.py 的 verdict-plan 与 verdict-record）；格数 × 每格条数
  mutation-units        crates 变异每行：读一次记录、写一次记录、写一份覆盖（crates-mutation-rows.py 的 judge_row 与 judge_coverage；用时表是运行记录，不进库）
  mutate-record-process mutate.sh 每条判完起一个 python 写一条（record_in_library）：每条一个进程
各家用的都是真的库类（lib_verdict_library.VerdictLibrary、admission.VerdictLibrary、lib_mutation_units.Library），库的本地目录放在这一趟的临时目录底下
（键空间照 scripts/prune-server/prune_server_client.py 文件头「键空间」带命名空间）。

用法（只在临时服务底下跑：客户端的 selftest_service_problem 不为空就拒，退 2）：
  python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary <prune-server 二进制> -- \
      python3 scripts/prune-server/prune-server-load-simulation.py [--count <每家条数，缺省 DEFAULT_COUNT>] [--cells <unit-verdicts 的格数，缺省 DEFAULT_CELLS>]
      每家一行 PRUNE_SERVER_SIMULATION family=… operations=… records=… seconds=… records_per_second=… failures=… host=…，
      末行 PRUNE_SERVER_SIMULATION_TOTAL families=… failures=… host=…。records 是这一家判定的条数，operations 是读写次数。
      退出码：0 每家都跑完、没有失败；1 有失败（服务不可用的那一家停在第一次失败，报 failures=1）；2 用法错（不在临时服务底下也算）。
  python3 scripts/prune-server/prune-server-load-simulation.py --selftest [--server-binary <prune-server 二进制>]
      二进制取法同客户端的 --selftest。

自证的格（CELLS）：
  every-family-runs          对着临时服务小量跑一趟：四家各一行、failures=0、退 0，本地临时库目录一个文件都没建
  failures-counted           对着一份 SIGSTOP 了的临时服务（回应限时 1 秒、重连 2 秒）：四家都报 failures 大于 0、退 1
  not-under-temporary-service 不设临时服务的标记：退 2，一行 PRUNE_SERVER_SIMULATION 都不打
弄坏开关 PRUNE_SERVER_SIMULATION_BREAK=<项>（只在自证里认）。一条一行「开关  弄坏哪一处」，下一行起写打开后红的格（括号里是失败行句首）：
  PRUNE_SERVER_SIMULATION_BREAK=failures-not-counted  服务不可用时不记失败、照报 0
      failures-counted（「✗ 自证 failures-counted：」）
"""
import collections
import importlib.util
import os
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
BREAK_VARIABLE = "PRUNE_SERVER_SIMULATION_BREAK"
SELFTEST_VARIABLE = "PRUNE_SERVER_SIMULATION_SELFTEST"
DEFAULT_COUNT = 2000
DEFAULT_CELLS = 4
MUTATE_PROCESS_SHARE = 20   # mutate.sh 每条一个进程，起进程比读写慢得多：照每家条数的二十分之一量
SELFTEST_COUNT = 40
SELFTEST_CELLS = 2
SELFTEST_RECONNECT_SECONDS = 2
FAMILIES = ("verdict-library", "unit-verdicts", "mutation-units", "mutate-record-process")
UnitKeyForSimulation = collections.namedtuple("UnitKeyForSimulation", "key composition refusals")
RequestForSimulation = collections.namedtuple("RequestForSimulation", "name")


def break_is(name):
    return os.environ.get(SELFTEST_VARIABLE) == "1" and name in os.environ.get(BREAK_VARIABLE, "").split(",")


def load_by_path(name, path):
    if name not in sys.modules:
        specification = importlib.util.spec_from_file_location(name, path)
        module = importlib.util.module_from_spec(specification)
        sys.modules[name] = module
        specification.loader.exec_module(module)
    return sys.modules[name]


def client_module():
    return load_by_path("singlefs_prune_server_client", os.path.join(SCRIPT_DIRECTORY, "prune_server_client.py"))


def harness_module():
    """客户端的自证那一份（起临时服务、找二进制）：只给 --selftest 用，不在模拟负载的正文里载入。"""
    return load_by_path("singlefs_prune_server_client_selftest", os.path.join(SCRIPT_DIRECTORY, "prune_server_client_selftest.py"))


def simulate_verdict_library(work, count):
    module = load_by_path("lib_verdict_library_for_simulation", os.path.join(REPOSITORY_ROOT, "scripts", "crash", "lib_verdict_library.py"))
    library = module.VerdictLibrary(os.path.join(work, "verdict-root", "singlefs-simulated-verdicts"), "simulated", module.digest_text("判法"))
    operations = 0
    for index in range(count):
        components = [("这一项", module.digest_text(f"第 {index} 项"))]
        library.lookup(f"项 {index}", components)
        library.record(f"项 {index}", components, "判绿", run="模拟负载")
        operations += 2
    return operations, count


def unit_verdicts_cell(work, cell, count):
    """一格（一个进程里）：逐条 lookup，再逐条 record。"""
    module = load_by_path("admission_for_simulation", os.path.join(REPOSITORY_ROOT, "scripts", "gate", "admission.py"))
    library = module.VerdictLibrary(work, f"simulated-gate.sh:cell-{cell}")
    keyed = [(RequestForSimulation(f"格 {cell} 第 {index} 条"), UnitKeyForSimulation(module.hashlib.sha256(f"{cell}:{index}".encode()).hexdigest(), (), ()))
             for index in range(count)]
    for request, unit_key in keyed:
        library.lookup(request, unit_key)
    for request, unit_key in keyed:
        library.record("请求原文", request.name, unit_key.key, "模拟负载")
    return 2 * count, count


def simulate_unit_verdicts(work, count, cells):
    tree = os.path.join(work, "unit-tree")
    os.makedirs(tree, exist_ok=True)
    subprocess.run(["git", "init", "-q", tree], check=True, capture_output=True)
    per_cell = max(1, count // cells)
    operations = records = 0
    for cell in range(cells):
        completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--unit-verdicts-cell", tree, str(cell), str(per_cell)],
                                   capture_output=True, text=True)
        if completed.returncode != 0:
            raise ChildFailed(completed.returncode, completed.stderr.strip()[-400:])
        cell_operations, cell_records = (int(word) for word in completed.stdout.split())
        operations += cell_operations
        records += cell_records
    return operations, records


def simulate_mutation_units(work, count):
    module = load_by_path("lib_mutation_units_for_simulation", os.path.join(REPOSITORY_ROOT, "scripts", "mutation", "lib_mutation_units.py"))
    library = module.Library(os.path.join(work, "mutation-root", "singlefs-simulated-mutation-library"))
    operations = 0
    for index in range(count):
        key = module.hashlib.sha256(f"行 {index}".encode()).hexdigest()
        library.read("records", key)
        library.write("records", key, {"kind": "mutation", "row_key": key, "verdict": "caught", "judged_date": "模拟"})
        library.write_json("coverage", key, {"units": [key[:16]]})
        operations += 3
    return operations, count


def simulate_mutate_record_process(work, count):
    directory = os.path.join(work, "mutate-root", "singlefs-simulated-mutate-library")
    operations = 0
    for index in range(max(1, count // MUTATE_PROCESS_SHARE)):
        completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--mutate-record", directory, str(index)], capture_output=True, text=True)
        if completed.returncode != 0:
            raise ChildFailed(completed.returncode, completed.stderr.strip()[-400:])
        operations += 1
    return operations, operations


class ChildFailed(Exception):
    def __init__(self, code, detail):
        super().__init__(f"子进程退 {code}：{detail}")
        self.code = code


def run_simulation(count, cells):
    client = client_module()
    problem = client.selftest_service_problem()
    if problem:
        print(f"  ✗ 压测只对临时服务跑：{problem}")
        print(f"     → 怎么办：{client.SELFTEST_SERVICE_REMEDY}（文件头「用法」）")
        return 2
    work = tempfile.mkdtemp(prefix="prune-simulation-")
    host = socket.gethostname()
    failures_total = 0
    try:
        runners = {"verdict-library": lambda: simulate_verdict_library(work, count),
                   "unit-verdicts": lambda: simulate_unit_verdicts(work, count, cells),
                   "mutation-units": lambda: simulate_mutation_units(work, count),
                   "mutate-record-process": lambda: simulate_mutate_record_process(work, count)}
        for family in FAMILIES:
            started = time.monotonic()
            failures, operations, records = 0, 0, 0
            try:
                operations, records = runners[family]()
            except (client.PruneServerFailure, ChildFailed) as error:
                failures = 0 if break_is("failures-not-counted") else 1
                print(f"  ! {family} 停在第一次失败：{error}")
            seconds = time.monotonic() - started
            failures_total += failures
            print(f"PRUNE_SERVER_SIMULATION family={family} operations={operations} records={records} seconds={seconds:.2f} "
                  f"records_per_second={records / seconds if seconds > 0 else 0:.1f} failures={failures} host={host}")
        local = [os.path.join(walked, name) for walked, _directories, names in os.walk(work) for name in names
                 if "unit-tree" not in os.path.relpath(walked, work).split(os.sep)[0]]
        print(f"PRUNE_SERVER_SIMULATION_TOTAL families={len(FAMILIES)} failures={failures_total} local_library_files={len(local)} host={host}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    return 1 if failures_total else 0


# ── 自证 ──

def selftest(arguments):
    os.environ[SELFTEST_VARIABLE] = "1"
    client = client_module()
    harness = harness_module()
    binary, problem = harness.resolve_binary(arguments)
    if binary is None or not os.access(binary, os.X_OK):
        print(f"  ✗ 自证起不了服务：{problem or binary + ' 不可执行'}")
        print(f"     → 怎么办：--server-binary <prune-server 二进制> 或 {client.BINARY_VARIABLE}=<路径>")
        return 2
    failed = []
    work = harness.selftest_root()
    server = harness.TestServer(binary, os.path.join(work, "server"))
    stopped = harness.TestServer(binary, os.path.join(work, "stopped-server"))
    try:
        base = {name: value for name, value in os.environ.items() if name not in harness.ENVIRONMENT_CLEARED_FOR_CELLS}

        def under(test_server):
            # 照 --with-temporary-service 设的那三个：连哪一处与临时服务的标记都指在这一份上
            return dict(base, **{client.SOCKET_VARIABLE: test_server.socket, client.TCP_ADDRESS_VARIABLE: test_server.tcp_address,
                                 client.TEMPORARY_SERVICE_VARIABLE: test_server.socket})

        command = [sys.executable, os.path.realpath(__file__), "--count", str(SELFTEST_COUNT), "--cells", str(SELFTEST_CELLS)]
        green = subprocess.run(command, capture_output=True, text=True, env=under(server))
        lines = [line for line in green.stdout.splitlines() if line.startswith("PRUNE_SERVER_SIMULATION family=")]
        passed = (green.returncode == 0 and len(lines) == len(FAMILIES) and all(" failures=0 " in line for line in lines)
                  and "local_library_files=0 " in green.stdout)
        print(f"  {'✓' if passed else '✗'} 自证 every-family-runs：退 {green.returncode}，{len(lines)} 行；{green.stdout.strip()[-600:]} {green.stderr.strip()[-300:]}")  # gate-lint:detail
        if not passed:
            failed.append("every-family-runs")
        # 服务进程组整组停住：socket 在、连得上，回应不来，客户端照回应限时与重连上限退 75
        stopped.signal_group(signal.SIGSTOP)
        try:
            dead = subprocess.run(command, capture_output=True, text=True,
                                  env=dict(under(stopped), **{client.RESPONSE_TIMEOUT_VARIABLE: "1",
                                                              client.RECONNECT_LIMIT_VARIABLE: str(SELFTEST_RECONNECT_SECONDS)}))
        finally:
            stopped.signal_group(signal.SIGCONT)
        lines = [line for line in dead.stdout.splitlines() if line.startswith("PRUNE_SERVER_SIMULATION family=")]
        passed = dead.returncode == 1 and len(lines) == len(FAMILIES) and all(" failures=0 " not in line for line in lines)
        print(f"  {'✓' if passed else '✗'} 自证 failures-counted：退 {dead.returncode}，{lines} {dead.stderr.strip()[-300:]}")  # gate-lint:detail
        if not passed:
            failed.append("failures-counted")
        outside = subprocess.run(command, capture_output=True, text=True, env=dict(under(server), **{client.TEMPORARY_SERVICE_VARIABLE: ""}))
        passed = outside.returncode == 2 and "PRUNE_SERVER_SIMULATION" not in outside.stdout
        print(f"  {'✓' if passed else '✗'} 自证 not-under-temporary-service：退 {outside.returncode}；{outside.stdout.strip()[-300:]}")  # gate-lint:detail
        if not passed:
            failed.append("not-under-temporary-service")
    finally:
        server.stop()
        stopped.stop()
        shutil.rmtree(work, ignore_errors=True)
    print(f"PRUNE_SERVER_SIMULATION_SELFTEST cells=3 passed={3 - len(failed)} failed={len(failed)} failed_cells={','.join(failed) or '-'}")
    if failed:
        print("  → 怎么办：照上面 ✗ 那几格后面的退出码与输出看模拟器与客户端；设着弄坏开关的，这就是要的红")
        return 1
    if os.environ.get(BREAK_VARIABLE):
        return 0
    swept = subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest", "--server-binary", binary], capture_output=True, text=True,
                           env=dict(os.environ, **{BREAK_VARIABLE: "failures-not-counted"}))
    matched = swept.returncode != 0 and "✗ 自证 failures-counted：" in swept.stdout
    print(f"PRUNE_SERVER_SIMULATION_BREAK_SWEEP switches=1 matched={int(matched)}")
    return 0 if matched else 1


def main(arguments):
    if os.environ.get(BREAK_VARIABLE) and os.environ.get(SELFTEST_VARIABLE) != "1" and arguments[:1] != ["--selftest"]:
        print(f"  ✗ {BREAK_VARIABLE} 只在 --selftest 里认，这一趟不是自证", file=sys.stderr)
        print(f"     → 怎么办：unset {BREAK_VARIABLE} 再跑", file=sys.stderr)
        return 2
    if arguments[:1] == ["--selftest"]:
        return selftest(arguments[1:])
    if arguments[:1] == ["--unit-verdicts-cell"] and len(arguments) == 4:
        operations, records = unit_verdicts_cell(arguments[1], int(arguments[2]), int(arguments[3]))
        print(operations, records)
        return 0
    if arguments[:1] == ["--mutate-record"] and len(arguments) == 3:
        module = load_by_path("lib_mutation_units_for_simulation", os.path.join(REPOSITORY_ROOT, "scripts", "mutation", "lib_mutation_units.py"))
        key = module.hashlib.sha256(f"变异 {arguments[2]}".encode()).hexdigest()
        module.Library(arguments[1]).write("records", key, {"kind": "run", "key": key, "verdict": "caught", "fails": "0"})
        return 0
    count, cells, position = DEFAULT_COUNT, DEFAULT_CELLS, 0
    while position < len(arguments):
        if arguments[position] in ("--count", "--cells") and position + 1 < len(arguments) and arguments[position + 1].isdigit() and int(arguments[position + 1]) > 0:
            if arguments[position] == "--count":
                count = int(arguments[position + 1])
            else:
                cells = int(arguments[position + 1])
            position += 2
            continue
        print(f"  ✗ 认不出的参数 {arguments[position]}")
        print("     → 怎么办：看文件头「用法」：--count <正整数>、--cells <正整数>")
        return 2
    return run_simulation(count, cells)


if __name__ == "__main__":
    sys.dont_write_bytecode = True
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..", ".claude", "scripts"))
    from project_preflight import preflight
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
