"""prune_server_client_selftest.py 的一块：对临时服务的各格（读写、前缀扫描、连接、分段大值、帧上限、临时服务随父进程退出与绑这一台的地址、命令行读与删除、崩溃判定库按结论种类整族删、写存储失败重放、GPU 行）。入口 prune_server_client_selftest.py 导入并再导出这里的名字。"""
import contextlib
import hashlib
import ipaddress
import json
import os
import random
import select
import signal
import socket
import subprocess
import sys
import threading
import time
from prune_server_client import (
    Client, EXIT_OTHER, EXIT_REFUSED, EXIT_UNAVAILABLE, EXIT_USAGE, FAMILIES, FAMILY_APPARATUS_ITEMS, FAMILY_UNIT_VERDICTS, GPU_CLAIM_FAMILY_PREFIX,
    LibraryStore, MAXIMUM_FRAME_BODY_BYTES, MULTI_HOST_CONFIGURATION_VARIABLE, PruneServerFailure, RECONNECT_LIMIT_VARIABLE,
    REPOSITORY_ROOT, RESPONSE_TIMEOUT_VARIABLE, SEGMENT_BYTES, SEGMENT_KEY_SEPARATOR, SOCKET_VARIABLE, STATUS_COMPLETED,
    TCP_ADDRESS_VARIABLE, TEMPORARY_SERVICE_VARIABLE, decode_gpu_pending_row, decode_gpu_rejudged_row, encode_gpu_pending_row,
    encode_manifest, gpu_claim_holder_identity, gpu_claim_key, gpu_conclusion_write, gpu_pending_key, gpu_pending_prefix_of_stream,
    gpu_rejudged_key, guard_row_state_of, manifest_of, manifest_version, namespace_of, parse_version_registry, process_client, segment_key,
    segment_key_parts, segment_version_prefix, segments_needed, selftest_service_problem, version_registry_key, conclusion_source_problems
)
from prune_server_client_selftest_service import (
    CHILD_TIMEOUT_SECONDS, ENTRY_SCRIPT_PATH, RUN_CONDITION_EXIT, SERVER_READY_SECONDS, STOP_EXIT_WITHIN_SECONDS, TestServer,
    cell_environment, environment_for, files_under, load_by_path, run_child, tcp_environment, this_host_address, unix_environment
)


SELFTEST_RECONNECT_SECONDS = 3
SELFTEST_BUSY_COMMIT_DELAY_MILLISECONDS = 200
SELFTEST_BUSY_RETRY_MILLISECONDS = 50

CONNECT_CELL_RECONNECT_SECONDS = 2     # connect-bounded-by-reconnect-total：重连总时长上限
CONNECT_CELL_RESPONSE_SECONDS = 30     # 同一格：等回应的超时，远大于总时长上限（建连给满它就堵过限时）
CONNECT_CELL_GRACE_SECONDS = 3         # 同一格：总时长上限之外容的秒数（起 python 子进程、退避抖动）
CONNECT_CELL_FILL_ATTEMPTS = 64        # 同一格：塞连接队列至多试这么多条
CONNECT_CELL_PROBE_SECONDS = 0.5       # 同一格：一条 TCP 连接这么久还没握手完成，就算队列满了
LARGE_VALUE_BYTES = MAXIMUM_FRAME_BODY_BYTES + 3 * 1024 * 1024 + 17   # large-value-round-trip：大过帧长上限的值（分 5 段）
LARGE_VALUE_SEED = 19
REWRITTEN_TAIL_BYTES = 5               # 同一格：重写成 SEGMENT_BYTES 加这么多字节（要两段）
INCOMPLETE_TAIL_BYTES = 11             # segmented-read-incomplete-refused：值是两整段加这么多字节（分 3 段）
OVER_LIMIT_VALUE_COUNT = 5             # batched-read-answer-over-frame-limit-split：各正好一段长（不分段存）的值几把，合计超过帧长上限
SHAPE_CELL_PREFIX = b"\x1bshape-cell/"  # segment-key-shapes-only-reserved：键都以一个表标签那样的字节起头（崩溃判定库的表标签在 0x00 到 0x1B）
SHAPE_CELL_TAIL_BYTES = 13             # 同一格：带 NUL 的那把大值是一整段加这么多字节（分 2 段）
OVERSIZE_EXIT_WITHIN_SECONDS = SELFTEST_RECONNECT_SECONDS   # oversize-frame-reported-at-once：要在重连总时长之内就退出（不重连）
PARENT_DEATH_EXIT_WITHIN_SECONDS = 10  # temporary-service-exits-with-parent：起服务的进程被 SIGKILL 之后，服务要在这么多秒里不在

CLIENT_PATH = os.path.join(os.path.dirname(os.path.realpath(__file__)), "prune_server_client.py")


def cell_round_trip_unix(binary, work):
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        with environment_for(unix_environment(server)):
            client = process_client()
            client.write_many([(b"round/one", b"1"), (b"round/two", b"2"), (b"round/three", b"\x00\xff")])
            values = client.read_many([b"round/one", b"round/two", b"round/three", b"round/missing"])
            written, already = client.write_frame([(b"round/one", b"overwritten-if-reapplied")], request_number=1)
            after = client.read_many([b"round/one"])[0]
            route = client.route
        passed = values == [b"1", b"2", b"\x00\xff", None] and already and after == b"1" and route.startswith("unix:")
        return passed, f"读回 {values}，同一请求号再发：写了 {written} 条、早已生效 {already}，之后 round/one={after!r}，走的 {route}"
    finally:
        server.stop()


def cell_prefix_scan(binary, work):
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        with environment_for(unix_environment(server)):
            client = process_client()
            client.write_many([(b"scan/a/2", b"two"), (b"scan/a/1", b"one"), (b"scan/b/1", b"other"), (b"scan/a/3", b"three")])
            entries = client.scan(b"scan/a/")
        passed = entries == [(b"scan/a/1", b"one"), (b"scan/a/2", b"two"), (b"scan/a/3", b"three")]
        return passed, f"扫 scan/a/ 得 {entries}"
    finally:
        server.stop()


def cell_tcp_round_trip(binary, work):
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        with environment_for(tcp_environment(server)):
            client = process_client()
            client.write_many([(b"tcp/one", b"over-tcp")])
            value = client.read_many([b"tcp/one"])[0]
            route = client.route
        return value == b"over-tcp" and route.startswith("tcp:"), f"读回 {value!r}，走的 {route}"
    finally:
        server.stop()


def cell_busy_retried_after_delay(binary, work):
    server = TestServer(binary, os.path.join(work, "server"),
                        ("--maximum-pending-total", "1", "--busy-retry-milliseconds", str(SELFTEST_BUSY_RETRY_MILLISECONDS),
                         "--selftest-commit-delay-milliseconds", str(SELFTEST_BUSY_COMMIT_DELAY_MILLISECONDS)))
    try:
        with environment_for(unix_environment(server)):
            clients = [Client(), Client()]
            problems = []

            def writer(client, label):
                try:
                    for index in range(4):
                        client.write_many([(f"busy/{label}/{index}".encode(), label.encode())])
                except PruneServerFailure as error:
                    problems.append(str(error))

            threads = [threading.Thread(target=writer, args=(client, label)) for client, label in zip(clients, ("left", "right"))]
            for thread in threads:
                thread.start()
            for thread in threads:
                thread.join()
            landed = process_client().scan(b"busy/")
            for client in clients:
                client.close()
        waits = [wait for client in clients for wait in client.busy_waits]
        short = [(milliseconds, seconds) for milliseconds, seconds in waits if seconds < milliseconds / 1000.0]
        busy = sum(client.busy_responses for client in clients)
        passed = not problems and len(landed) == 8 and busy >= 1 and not short
        return passed, f"忙 {busy} 次，等得不够的 {short[:3]}，落库 {len(landed)} 条（要 8），出错 {problems[:2]}"
    finally:
        server.stop()


def cell_stopped_exits_75(binary, work):
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        status = server.announce_stop()
        code, seconds, output, error = run_child(["read", "anything"], dict(unix_environment(server), **{RECONNECT_LIMIT_VARIABLE: "30"}),
                                                 timeout=STOP_EXIT_WITHIN_SECONDS * 2)
        passed = status == STATUS_COMPLETED and code == EXIT_UNAVAILABLE and seconds < STOP_EXIT_WITHIN_SECONDS and "服务停了" in error
        return passed, (f"宣布停服务回状态 {status}；子进程退 {code}（{seconds:.1f} 秒，要 {STOP_EXIT_WITHIN_SECONDS} 秒内），"
                        f"标准输出「{output.strip()}」，标准错误「{error.strip()[-300:]}」")
    finally:
        server.stop()


def cell_unreachable_exits_75(binary, work):
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        with environment_for(unix_environment(server)):
            process_client().write_many([(b"unreachable/one", b"1")])
        server.kill()
        code, seconds, output, error = run_child(["read", "unreachable/one"],
                                                 dict(unix_environment(server), **{RECONNECT_LIMIT_VARIABLE: str(SELFTEST_RECONNECT_SECONDS)}),
                                                 timeout=SELFTEST_RECONNECT_SECONDS * 8)
        passed = code == EXIT_UNAVAILABLE and "连不上" in error and "absent" not in output
        return passed, f"子进程退 {code}（{seconds:.1f} 秒；None 是超时被停），标准输出「{output.strip()}」，标准错误「{error.strip()[-300:]}」"
    finally:
        server.stop()


def cell_families_over_service(binary, work):
    """五族（三家、广谱变异的驱动、装置的转交）各开一份库目录在临时目录底下的库，读写一遍；服务侧按各自前缀扫、本地目录一个文件都没有。
    装置的转交（apparatus-items）整条路由 device-code-key.py 的自证核，这里只经 LibraryStore 写一条、读回、看前缀。"""
    server = TestServer(binary, os.path.join(work, "server"))
    notes, passed = [], True
    try:
        verdict_directory = os.path.join(work, "verdict-root", "singlefs-sample-verdicts")
        mutation_directory = os.path.join(work, "mutation-root", "singlefs-sample-library")
        broad_directory = os.path.join(work, "broad-root", "broad-mutation-units")
        admission_tree = os.path.join(work, "admission-tree")
        os.makedirs(admission_tree)
        initialized = subprocess.run(["git", "init", "-q", admission_tree], capture_output=True, text=True, timeout=CHILD_TIMEOUT_SECONDS)
        admission_common = os.path.join(admission_tree, ".git")
        before_admission = files_under(admission_common)
        with environment_for(unix_environment(server)):
            verdict_module = load_by_path("lib_verdict_library_for_prune_client_cells", os.path.join(REPOSITORY_ROOT, "scripts", "crash", "lib_verdict_library.py"))
            library = verdict_module.VerdictLibrary(verdict_directory, "kind", verdict_module.digest_text("判法"))
            components = [("这一项", verdict_module.digest_text("甲"))]
            first = library.lookup("一项", components)
            library.record("一项", components, "判绿", run="自证")
            second = library.lookup("一项", components)
            library.record_red("一项", components, "判红", run="自证")
            third = library.lookup("一项", components)
            verdict_ok = first == (False, verdict_module.NO_RECORD_REASON) and second[0] and third == (False, verdict_module.RED_RECORD_REASON)
            notes.append(f"verdict-library：头一次 {first}、记绿之后 {second}、记红之后 {third}")

            admission_module = load_by_path("admission_for_prune_client_cells", os.path.join(REPOSITORY_ROOT, "scripts", "gate", "admission.py"))
            unit_library = admission_module.VerdictLibrary(admission_tree, "sample-gate.sh:cell")
            problem = unit_library.record("请求原文", "一条", "a" * 64, "自证")
            items = unit_library.items()
            read_back = unit_library.read(unit_library.path_of_key("a" * 64))
            unit_library.record_red("请求原文", "一条", "a" * 64, "自证")
            red_back = unit_library.read_red(unit_library.path_of_key("a" * 64), "a" * 64)
            admission_ok = problem == "" and [name for name, _record in items] == ["一条"] and (read_back or {}).get("verdict") == "green" \
                and (red_back or {}).get("verdict") == "red"
            notes.append(f"unit-verdicts：写 {problem!r}、items {[name for name, _record in items]}、读回 {(read_back or {}).get('verdict')}、记红之后 {(red_back or {}).get('verdict')}")

            units_module = load_by_path("lib_mutation_units_for_prune_client_cells", os.path.join(REPOSITORY_ROOT, "scripts", "mutation", "lib_mutation_units.py"))
            mutation_library = units_module.Library(mutation_directory)
            mutation_library.write("records", "row-key", {"kind": "mutation", "verdict": "caught"})
            mutation_library.write_json("coverage", "coverage-key", {"units": ["abc"]})
            record = mutation_library.read("records", "row-key")
            coverage = mutation_library.read_json("coverage", "coverage-key")
            mutation_ok = (record or {}).get("verdict") == "caught" and coverage == {"units": ["abc"]}
            notes.append(f"mutation-units：记录 {record}、覆盖 {coverage}")

            broad_module = load_by_path("broad_mutation_for_prune_client_cells", os.path.join(REPOSITORY_ROOT, "scripts", "mutation", "broad-mutation.py"))
            broad_library = broad_module.Library(broad_directory)
            broad_library.record_verdict(["a" * 64, "1", "singlefs-core", "自证的一条", "caught", "one", "自证的测试", "b" * 64, "", "0.0", "0.0",
                                          broad_module.today_text(), "fresh", "", "", ""])
            broad_outcomes = [row["outcome"] for row in broad_library.read_verdicts().rows]
            broad_ok = broad_outcomes == ["caught"]
            notes.append(f"broad-mutation：读回的结局 {broad_outcomes}")

            apparatus_store = LibraryStore(FAMILY_APPARATUS_ITEMS, "fake-device", os.path.join(work, "apparatus-root"))
            apparatus_store.write("items/item-key.out", b"judged\n")
            apparatus_ok = apparatus_store.read("items/item-key.out") == b"judged\n"
            notes.append(f"apparatus-items：读回{'对' if apparatus_ok else '不对'}")

            client = process_client()
            prefixes = {family: LibraryStore(family, "", os.path.join(work, "x")).prefix.split(b"@")[0].rstrip(b"/") for family in FAMILIES}
            scanned = {family: [key for key, _value in client.scan(prefixes[family])] for family in FAMILIES}
        distinct = len(set(prefixes.values())) == len(FAMILIES)
        each_own = all(scanned[family] and all(key.startswith(prefixes[family] + b"@") for key in scanned[family]) for family in FAMILIES) \
            and sum(len(keys) for keys in scanned.values()) == len({key for keys in scanned.values() for key in keys})
        local_files = files_under(verdict_directory) + files_under(mutation_directory) + files_under(broad_directory) \
            + [path for path in files_under(admission_common) if path not in before_admission]
        passed = initialized.returncode == 0 and verdict_ok and admission_ok and mutation_ok and broad_ok and apparatus_ok and distinct and each_own and not local_files
        notes.append(f"五族前缀 {sorted(prefixes.values())}（互不相同 {distinct}），各自扫到 { {family: len(keys) for family, keys in scanned.items()} }、"
                     f"都只在自己前缀底下 {each_own}，本地新建的文件 {local_files[:5]}")
        return passed, "；".join(notes)
    finally:
        server.stop()


def cell_unreachable_family_exits_75(binary, work):
    """服务连不上（指到一个没人听的回环端口）：lib_verdict_library 的 lookup 子进程退 75、说连不上，不打「没给判定库」「无记录」那一对，库目录没建。"""
    library_directory = os.path.join(work, "unreachable-root", "singlefs-sample-verdicts")
    closed = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    closed.bind(("127.0.0.1", 0))
    address = "127.0.0.1:%d" % closed.getsockname()[1]
    closed.close()
    code, seconds, output, error = run_child(["verdict-lookup", library_directory],
                                             {SOCKET_VARIABLE: os.path.join(work, "no-such.sock"), TCP_ADDRESS_VARIABLE: address,
                                              RECONNECT_LIMIT_VARIABLE: str(SELFTEST_RECONNECT_SECONDS)}, timeout=SELFTEST_RECONNECT_SECONDS * 8)
    local = files_under(os.path.dirname(library_directory))
    passed = code == EXIT_UNAVAILABLE and not local and "连不上" in error and not output.strip()
    return passed, f"子进程退 {code}（{seconds:.1f} 秒），库目录底下 {local[:3]}，标准输出「{output.strip()}」，标准错误「{error.strip()[-300:]}」"


def filled_unix_listener(path):
    """只听不收的 unix socket，连接队列塞满（再来一条非阻塞连接当场回 EAGAIN 才停）：交 (监听的 socket, 塞进去的连接)。"""
    listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    listener.bind(path)
    listener.listen(1)
    fillers = []
    for _ in range(CONNECT_CELL_FILL_ATTEMPTS):
        filler = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        filler.setblocking(False)
        try:
            filler.connect(path)
        except BlockingIOError:
            filler.close()
            return listener, fillers
        fillers.append(filler)
    raise RuntimeError(f"unix socket 塞了 {CONNECT_CELL_FILL_ATTEMPTS} 条连接，队列还没满")


def filled_tcp_listener():
    """只听不收的回环 TCP，连接队列塞满（再来一条连接 CONNECT_CELL_PROBE_SECONDS 秒里握手不完成才停，SYN 被丢）：
    交 (监听的 socket, 塞进去的连接, 「地址:端口」)。"""
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    host, port = listener.getsockname()
    fillers = []
    for _ in range(CONNECT_CELL_FILL_ATTEMPTS):
        probe = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        probe.setblocking(False)
        probe.connect_ex((host, port))
        _readable, writable, _failed = select.select([], [probe], [], CONNECT_CELL_PROBE_SECONDS)
        if writable and probe.getsockopt(socket.SOL_SOCKET, socket.SO_ERROR) == 0:
            fillers.append(probe)
            continue
        probe.close()
        return listener, fillers, f"{host}:{port}"
    raise RuntimeError(f"回环 TCP 塞了 {CONNECT_CELL_FILL_ATTEMPTS} 条连接，每一条都握手完成了，队列还没满")


def cell_connect_bounded_by_reconnect_total(binary, work):
    """建连只用重连总时长剩下的时间（客户端文件头「容错」）：unix 连接队列满、TCP 握手不完成两种现场，子进程读一次都在总时长加
    CONNECT_CELL_GRACE_SECONDS 秒里退 75、说连不上。两种现场都是这一格自己听着不收的 socket，不起服务（binary 不用）。"""
    del binary
    changes = {RECONNECT_LIMIT_VARIABLE: str(CONNECT_CELL_RECONNECT_SECONDS), RESPONSE_TIMEOUT_VARIABLE: str(CONNECT_CELL_RESPONSE_SECONDS)}
    within = CONNECT_CELL_RECONNECT_SECONDS + CONNECT_CELL_GRACE_SECONDS
    opened, notes, passed = [], [], True
    try:
        unix_path = os.path.join(work, "full.sock")
        unix_listener, unix_fillers = filled_unix_listener(unix_path)
        opened += [unix_listener, *unix_fillers]
        tcp_listener, tcp_fillers, tcp_address = filled_tcp_listener()
        opened += [tcp_listener, *tcp_fillers]
        routes = (("unix 连接队列满", len(unix_fillers), {SOCKET_VARIABLE: unix_path}),
                  ("TCP 握手不完成", len(tcp_fillers), {SOCKET_VARIABLE: os.path.join(work, "no-such.sock"), TCP_ADDRESS_VARIABLE: tcp_address}))
        for what, filled, route in routes:
            code, seconds, output, error = run_child(["read", "connect/any"], dict(changes, **route), timeout=within)
            route_passed = code == EXIT_UNAVAILABLE and seconds < within and "连不上" in error and "absent" not in output
            passed = passed and route_passed
            notes.append(f"{what}（队列里塞了 {filled} 条）：子进程退 {code}（{seconds:.1f} 秒，要 {within} 秒内；None 是超时被停），"
                         f"标准错误「{error.strip()[-200:]}」")
        return passed, "；".join(notes)
    finally:
        for opened_socket in opened:
            with contextlib.suppress(OSError):
                opened_socket.close()


def stored_segments(raw_entries, key):
    """服务里原样扫回的 [(键, 值)] 里这把键带版本的段：{版本: [段号…升序]}（客户端文件头「键空间」段键 <键> NUL <版本> NUL <段号>）。"""
    owner = key + SEGMENT_KEY_SEPARATOR
    found = {}
    for raw_key, _value in raw_entries:
        parts = segment_key_parts(raw_key)
        if parts is not None and parts[0].startswith(owner):
            found.setdefault(parts[0][len(owner):], []).append(parts[1])
    return {version: sorted(indices) for version, indices in found.items()}


def version_of(value):
    """这个整值存成分段时的版本（弄坏开关 segments-overwritten-in-place 下是 None，段键不带版本）。"""
    return manifest_version({"sha256": hashlib.sha256(value).hexdigest()})


def cell_large_value_round_trip(binary, work):
    """大于帧长上限的值经分段写读往返、扫描只交原键；重写成两段长的值之后服务里只剩新的那一版段、版本登记键只登记它（客户端文件头「键空间」大值分段）。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        large = random.Random(LARGE_VALUE_SEED).randbytes(LARGE_VALUE_BYTES)
        rewritten = large[:SEGMENT_BYTES + REWRITTEN_TAIL_BYTES]
        with environment_for(unix_environment(server)):
            client = process_client()
            client.write_many([(b"large/one", large), (b"large/small", b"small")])
            values = client.read_many([b"large/one", b"large/small", b"large/missing"])
            round_trip_ok = values[0] == large and values[1:] == [b"small", None]
            scanned = client.scan(b"large/")
            scanned_keys = [key for key, _value in scanned]
            scan_ok = scanned_keys == [b"large/one", b"large/small"] and scanned[0][1] == large
            del values, scanned
            before = stored_segments(client.raw_scan_stream(b"large/one"), b"large/one")
            client.write_many([(b"large/one", rewritten)])
            after = client.read_many([b"large/one"])[0]
            raw_after = dict(client.raw_scan_stream(b"large/one"))
        manifest = manifest_of(b"large/one", raw_after.get(b"large/one"))
        listed = len(manifest["segments"]) if manifest else 0
        after_segments = stored_segments(raw_after.items(), b"large/one")
        registered = parse_version_registry(b"large/one", raw_after.get(version_registry_key(b"large/one")))
        large_version, rewritten_version = version_of(large), version_of(rewritten)
        before_ok = before == {large_version: list(range(segments_needed(large)))}
        rewrite_ok = (after == rewritten and manifest is not None and after_segments == {rewritten_version: list(range(listed))}
                      and registered == [rewritten_version])
        passed = round_trip_ok and scan_ok and before_ok and rewrite_ok
        shown = {version.decode()[:12]: indices for version, indices in after_segments.items()}
        return passed, (f"写读 {len(large)} 字节的值逐字节相同 {round_trip_ok}（小值与不在的键一并对 {round_trip_ok}）；按前缀扫交 {scanned_keys}、大值拼回相同 {scan_ok}；"
                        f"服务里的段 {[(version.decode()[:12], indices) for version, indices in before.items()]}（要只有这个值那一版、段号 0 到 {segments_needed(large) - 1}）；"
                        f"重写成 {len(rewritten)} 字节之后读回相同 {after == rewritten}、新清单点名 {listed} 段、服务里的段 {shown}、"
                        f"版本登记 {[version.decode()[:12] for version in registered]}（要只剩新值那一版 {rewritten_version.decode()[:12] if rewritten_version else None}、"
                        "段号与清单一一对上、旧的那一版回收了）")
    finally:
        server.stop()


def segmented_read_outcome(action):
    """跑一次读：交「退 3（segmented-read）」这类说法与它是不是分段读错。"""
    try:
        result = action()
    except PruneServerFailure as failure:
        return f"退 {failure.code}（{failure.kind}）", failure.kind == "segmented-read"
    if isinstance(result, list):
        result = result[0] if result else None
    return ("交回「不在」" if result is None else f"交回 {len(result) if isinstance(result, bytes) else result} 字节"), False


def cell_segmented_read_incomplete_refused(binary, work):
    """分段存的值拼不全（缺段）、一段短了、一段同长换了字节：点读都退 3（分段拼不回），按前缀扫换了字节的那一把同样退 3。
    缺段那一把是另起一把键、只写清单与这一版的段号 0、2。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        value = random.Random(LARGE_VALUE_SEED + 1).randbytes(SEGMENT_BYTES * 2 + INCOMPLETE_TAIL_BYTES)
        version = version_of(value)
        pieces = [value[:SEGMENT_BYTES], value[SEGMENT_BYTES:SEGMENT_BYTES * 2], value[SEGMENT_BYTES * 2:]]
        altered = bytes([pieces[1][0] ^ 0xFF]) + pieces[1][1:]
        with environment_for(unix_environment(server)):
            client = process_client()
            client.write_many([(b"broken/short", value), (b"broken/altered", value)])
            client.write_frame([(b"broken/gap", encode_manifest(value, [len(piece) for piece in pieces])),
                                (segment_key(b"broken/gap", version, 0), pieces[0]), (segment_key(b"broken/gap", version, 2), pieces[2])])
            client.write_frame([(segment_key(b"broken/short", version, 2), pieces[2][:-1])])
            client.write_frame([(segment_key(b"broken/altered", version, 1), altered)])
            outcomes = {name: segmented_read_outcome(lambda key=key: client.read_many([key]))
                        for name, key in (("缺段", b"broken/gap"), ("短段", b"broken/short"), ("换了字节", b"broken/altered"))}
            outcomes["按前缀扫换了字节的"] = segmented_read_outcome(lambda: [value for _key, value in client.scan(b"broken/altered")])
        passed = all(refused for _shown, refused in outcomes.values())
        return passed, "；".join(f"{name}：{shown}" for name, (shown, _refused) in outcomes.items()) + "（每一种都要退 3（segmented-read））"
    finally:
        server.stop()


REWRITE_MANIFEST_AFTER_SECONDS = 0.3


def cell_segmented_read_sees_a_rewrite(binary, work):
    """同一把键正被重写（客户端文件头「键空间」大值分段「读」）：新一版的段已写下、读的一方要读的旧一版段已被回收、清单还是旧的，
    另一条线程 REWRITE_MANIFEST_AFTER_SECONDS 秒后写新清单；点读与按前缀扫都交回新值，不退 3。"""
    server = TestServer(binary, os.path.join(work, "server"))
    old = random.Random(LARGE_VALUE_SEED + 3).randbytes(SEGMENT_BYTES + INCOMPLETE_TAIL_BYTES)
    new = random.Random(LARGE_VALUE_SEED + 4).randbytes(SEGMENT_BYTES + INCOMPLETE_TAIL_BYTES)
    lengths = [SEGMENT_BYTES, INCOMPLETE_TAIL_BYTES]
    old_version, new_version = version_of(old), version_of(new)
    try:
        with environment_for(unix_environment(server)):
            client = process_client()
            outcomes = {}
            for name, key, action in (("点读", b"rewritten/read", lambda key: client.read_many([key])[0]),
                                      ("按前缀扫", b"rewritten/scan", lambda key: dict(client.scan(key)).get(key))):
                client.write_many([(key, old)])
                client.write_frame([(segment_key(key, new_version, 0), new[:SEGMENT_BYTES]), (segment_key(key, new_version, 1), new[SEGMENT_BYTES:])])
                if old_version is not None:
                    client.delete_prefix(segment_version_prefix(key, old_version))
                writer = threading.Timer(REWRITE_MANIFEST_AFTER_SECONDS, lambda key=key: client.write_frame([(key, encode_manifest(new, lengths))]))
                writer.start()
                try:
                    result = action(key)
                    outcomes[name] = ("交回新值" if result == new else "交回旧值" if result == old else
                                      "交回「不在」" if result is None else f"交回 {len(result)} 字节的别的值")
                except PruneServerFailure as failure:
                    outcomes[name] = f"退 {failure.code}（{failure.kind}）"
                finally:
                    writer.join()
        passed = all(shown == "交回新值" for shown in outcomes.values())
        return passed, "；".join(f"{name}：{shown}" for name, shown in outcomes.items()) + "（都要交回新值）"
    finally:
        server.stop()


DEATH_KEY = b"death/one"
DEATH_NEW_SEED = LARGE_VALUE_SEED + 6


def cell_segmented_rewrite_survives_writer_death(binary, work):
    """写方死在「写完新一版的段、还没写清单」之间（客户端文件头「键空间」大值分段「写」）：子进程照 write_many 写新值，写到清单那一帧之前 SIGKILL 自己；
    点读与按前缀扫都交回旧值、不退 3，服务里登记着旧新两版；再写第三个值之后读回它，段与版本登记键都只剩第三个值那一版。"""
    server = TestServer(binary, os.path.join(work, "server"))
    old = random.Random(LARGE_VALUE_SEED + 5).randbytes(SEGMENT_BYTES + INCOMPLETE_TAIL_BYTES)
    new = random.Random(DEATH_NEW_SEED).randbytes(SEGMENT_BYTES + INCOMPLETE_TAIL_BYTES)
    third = random.Random(LARGE_VALUE_SEED + 7).randbytes(SEGMENT_BYTES + REWRITTEN_TAIL_BYTES)
    try:
        with environment_for(unix_environment(server)):
            process_client().write_many([(DEATH_KEY, old)])
        code, _seconds, output, error = run_child(["write-dies-before-manifest", DEATH_KEY.decode(), str(DEATH_NEW_SEED), str(len(new))],
                                                  unix_environment(server))
        killed = code == -signal.SIGKILL and "written" not in output
        outcomes = {}
        with environment_for(unix_environment(server)):
            client = process_client()
            for name, action in (("点读", lambda: client.read_many([DEATH_KEY])[0]), ("按前缀扫", lambda: dict(client.scan(b"death/")).get(DEATH_KEY))):
                try:
                    result = action()
                    outcomes[name] = ("交回旧值" if result == old else "交回新值" if result == new else
                                      "交回「不在」" if result is None else f"交回 {len(result)} 字节的别的值")
                except PruneServerFailure as failure:
                    outcomes[name] = f"退 {failure.code}（{failure.kind}）"
            raw_after_death = dict(client.raw_scan_stream(DEATH_KEY))
            registered_after_death = parse_version_registry(DEATH_KEY, raw_after_death.get(version_registry_key(DEATH_KEY)))
            del raw_after_death
            client.write_many([(DEATH_KEY, third)])
            third_read = client.read_many([DEATH_KEY])[0]
            raw_third = dict(client.raw_scan_stream(DEATH_KEY))
        segments_third = stored_segments(raw_third.items(), DEATH_KEY)
        registered_third = parse_version_registry(DEATH_KEY, raw_third.get(version_registry_key(DEATH_KEY)))
        both = sorted(version for version in (version_of(old), version_of(new)) if version is not None)
        third_version = version_of(third)
        reads_ok = all(shown == "交回旧值" for shown in outcomes.values())
        death_registry_ok = registered_after_death == both
        third_ok = (third_read == third and segments_third == {third_version: list(range(segments_needed(third)))}
                    and registered_third == [third_version])
        passed = killed and reads_ok and death_registry_ok and third_ok
        return passed, (f"子进程写新值：退 {code}（要被 SIGKILL：{-signal.SIGKILL}）、标准输出「{output.strip()}」、标准错误「{error.strip()[-200:]}」；"
                        + "；".join(f"{name}：{shown}" for name, shown in outcomes.items()) + "（都要交回旧值、不退 3）；"
                        f"死后版本登记 {[version.decode()[:12] for version in registered_after_death]}（要旧新两版 {[version.decode()[:12] for version in both]}）；"
                        f"再写第三个值读回相同 {third_read == third}、服务里的段 {[(version.decode()[:12], indices) for version, indices in segments_third.items()]}、"
                        f"版本登记 {[version.decode()[:12] for version in registered_third]}（要只剩第三个值那一版）")
    finally:
        server.stop()


def cell_segment_count_over_index_digits_refused(binary, work):
    """段数段号装不下的值写之前拒（客户端文件头「键空间」大值分段「写」）：段长与段号位数调成 1 字节、1 位（至多 10 段），
    要 11 段的值退 3（too-many-segments）、原键、段键、版本登记键一条都没写；要 10 段的照写、读回相同。"""
    server = TestServer(binary, os.path.join(work, "server"))
    client_module = sys.modules[Client.__module__]
    saved = (client_module.SEGMENT_BYTES, client_module.SEGMENT_INDEX_DIGITS)
    eleven = b"0123456789A"
    try:
        with environment_for(unix_environment(server)):
            client = process_client()
            client_module.SEGMENT_BYTES, client_module.SEGMENT_INDEX_DIGITS = 1, 1
            try:
                refused = write_outcome(client, [(b"digits/eleven", eleven)])
                written = write_outcome(client, [(b"digits/ten", b"0123456789")])
                ten = client.read_many([b"digits/ten"])[0]
                left = client.read_many_raw([b"digits/eleven", client_module.version_registry_key(b"digits/eleven"),
                                             *[client_module.segment_key(b"digits/eleven", version_of(eleven), index) for index in range(11)]])
            finally:
                client_module.SEGMENT_BYTES, client_module.SEGMENT_INDEX_DIGITS = saved
        stored = [value for value in left if value is not None]
        passed = refused == "too-many-segments" and not stored and written == "written" and ten == b"0123456789"
        return passed, (f"11 段：{refused}、服务里留下 {len(stored)} 条（要 too-many-segments、0 条）；"
                        f"10 段：{written}、读回 {ten!r}（要照写、读回 b'0123456789'）")
    finally:
        client_module.SEGMENT_BYTES, client_module.SEGMENT_INDEX_DIGITS = saved
        server.stop()


SCAN_ROW_OVER_FRAME_CALL_LIMIT = 300


def cell_scan_row_over_frame_refused(_binary, _work):
    """按前缀扫一帧截断而一行都没回（客户端文件头「键空间」大值分段「扫」）：假的服务每一帧都回「截断、0 行」，问了一帧就退 3（scan-row-over-frame）。
    往下拆前缀的（改前的样子）问到 SCAN_ROW_OVER_FRAME_CALL_LIMIT 帧就停下，判红。"""
    calls = []

    class TruncatingEveryFrame(Client):
        def raw_scan(self, prefix, after_key=b""):
            calls.append(prefix)
            if len(calls) > SCAN_ROW_OVER_FRAME_CALL_LIMIT:
                raise RuntimeError(f"问了 {len(calls)} 帧还在往下拆前缀")
            return [], True

        def read_many_raw(self, keys):
            return [None for _key in keys]

    try:
        list(TruncatingEveryFrame().raw_scan_stream(b"over-frame/"))
        shown = "扫完了、没退"
        kind = None
    except PruneServerFailure as failure:
        shown, kind = f"退 {failure.code}（{failure.kind}）", failure.kind
    except (RuntimeError, RecursionError) as stopped:
        shown, kind = f"没退：{stopped}", None
    passed = kind == "scan-row-over-frame" and len(calls) == 1
    return passed, f"{shown}，问了 {len(calls)} 帧（要退 3（scan-row-over-frame）、只问 1 帧）"


def write_outcome(client, entries):
    """write_many 一批：交「written」或失败的种类（reserved-key 这类）。"""
    try:
        client.write_many(entries)
    except PruneServerFailure as failure:
        return failure.kind
    return "written"


def cell_segment_key_shapes_only_reserved(binary, work):
    """认段键只认形状（客户端文件头「键空间」认段键只认形状）：带 NUL 而不是段键、版本登记键形状的原键照写照读照扫，
    一把排在大值原键与它的段键之间的原键不打乱拼回；段键、版本登记键形状的原键写时整批拒。"""
    server = TestServer(binary, os.path.join(work, "server"))
    big_key = SHAPE_CELL_PREFIX + b"big\x00\x07"
    between_key = big_key + b"\x00\x01"
    big = random.Random(LARGE_VALUE_SEED + 2).randbytes(SEGMENT_BYTES + SHAPE_CELL_TAIL_BYTES)
    stored = {SHAPE_CELL_PREFIX + b"\x00\x00digest\x00": b"label-and-nul", SHAPE_CELL_PREFIX + b"five\x0012345": b"five-digits",
              SHAPE_CELL_PREFIX + b"seven\x001234567": b"seven-digits", SHAPE_CELL_PREFIX + b"listed\x00versionsmore": b"versions-then-more",
              big_key: big, between_key: b"between-owner-and-segments"}
    refused = {"段键形状": (SHAPE_CELL_PREFIX + b"refused\x00000003", SHAPE_CELL_PREFIX + b"alongside-segment-shape"),
               "版本登记键形状": (SHAPE_CELL_PREFIX + b"refused\x00versions", SHAPE_CELL_PREFIX + b"alongside-registry-shape")}
    try:
        with environment_for(unix_environment(server)):
            client = process_client()
            stored_outcome = write_outcome(client, list(stored.items()))
            read_ok = client.read_many(list(stored)) == list(stored.values())
            scanned = client.scan(SHAPE_CELL_PREFIX)
            scanned_keys = [key for key, _value in scanned]
            scan_ok = scanned == sorted(stored.items())
            del scanned
            raw_keys = [key for key, _value in client.raw_scan_stream(SHAPE_CELL_PREFIX)]
            refusals = {}
            for name, (reserved_key, alongside_key) in refused.items():
                kind = write_outcome(client, [(alongside_key, b"legal"), (reserved_key, b"shaped")])
                refusals[name] = (kind, client.read_many_raw([reserved_key, alongside_key]))
        big_segment_keys = [segment_key(big_key, version_of(big), index) for index in range(segments_needed(big))]
        stored_raw_ok = all(key in raw_keys for key in [*big_segment_keys, version_registry_key(big_key)])
        interleaved = (all(key in raw_keys for key in (big_key, between_key, big_segment_keys[0]))
                       and raw_keys.index(big_key) < raw_keys.index(between_key) < raw_keys.index(big_segment_keys[0]))
        refused_ok = all(kind == "reserved-key" and values == [None, None] for kind, values in refusals.values())
        passed = stored_outcome == "written" and read_ok and scan_ok and stored_raw_ok and interleaved and refused_ok
        return passed, (f"带 NUL 而不是那两种形状的 {len(stored)} 把原键：写 {stored_outcome}（要 written）、逐字节读回相同 {read_ok}、"
                        f"按前缀扫交 {scanned_keys}、与写进去的逐条相同 {scan_ok}；服务里原样有大值的段键与版本登记键 {stored_raw_ok}、"
                        f"排在原键与段键之间的那把确实夹在中间 {interleaved}；"
                        + "；".join(f"{name}：写 {kind}、服务里这把与同批合法那条 {['不在' if value is None else '在' for value in values]}"
                                   for name, (kind, values) in refusals.items())
                        + "（都要 reserved-key、两条都不在）")
    finally:
        server.stop()


def cell_oversize_frame_reported_at_once(binary, work):
    """单条超限当场报（客户端文件头「容错」）：子进程写键长超过帧长上限的一条，当场退 3、说帧长上限；进程内把本地上限放宽到两倍照发，
    服务回「帧长度不对」，当场退 3（frame-too-large）、不重连；之后服务照常可用。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        oversize = MAXIMUM_FRAME_BODY_BYTES + 1
        changes = dict(unix_environment(server), **{RECONNECT_LIMIT_VARIABLE: str(SELFTEST_RECONNECT_SECONDS)})
        code, seconds, output, error = run_child(["write-long-key", str(oversize)], changes, timeout=SELFTEST_RECONNECT_SECONDS * 8)
        child_ok = code == EXIT_OTHER and seconds < OVERSIZE_EXIT_WITHIN_SECONDS and "帧长上限" in error and "连不上" not in error and "written" not in output
        with environment_for(changes):
            client = Client()
            client.frame_limit = MAXIMUM_FRAME_BODY_BYTES * 2
            started = time.monotonic()
            try:
                client.write_frame([(b"o" * oversize, b"v")])
                server_side, server_kind = "写成了", ""
            except PruneServerFailure as failure:
                server_side, server_kind = failure.message[-300:], failure.kind
            finally:
                client.close()
            server_seconds = time.monotonic() - started
            process_client().write_many([(b"oversize/after", b"1")])
            usable = process_client().read_many([b"oversize/after"])[0] == b"1"
        server_ok = server_kind == "frame-too-large" and "帧长度不对" in server_side and server_seconds < OVERSIZE_EXIT_WITHIN_SECONDS
        passed = child_ok and server_ok and usable
        return passed, (f"子进程写 {oversize} 字节的键：退 {code}（{seconds:.1f} 秒，要 {OVERSIZE_EXIT_WITHIN_SECONDS} 秒内退 3），标准输出「{output.strip()}」，"
                        f"标准错误「{error.strip()[-300:]}」；本地上限放宽之后照发：{server_kind or '没报错'}（{server_seconds:.1f} 秒）「{server_side}」"
                        f"（要 frame-too-large、说帧长度不对）；之后服务照常读写 {usable}")
    finally:
        server.stop()


def process_state_and_start(process_id):
    """/proc/<进程号>/stat 的状态与起始时刻（时钟滴答，字段 3 与 22）：进程不在交 None。"""
    try:
        with open(f"/proc/{process_id}/stat", encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except OSError:
        return None
    fields = text[text.rindex(")") + 2:].split()
    return fields[0], fields[19]


def still_running(process_id, start_ticks):
    """这个进程号还是起始时刻为 start_ticks 的那一个、又不是僵尸：换成别的进程、成了僵尸、不在都算不在。"""
    state = process_state_and_start(process_id)
    return state is not None and state[0] not in ("Z", "X") and state[1] == start_ticks


def cell_temporary_service_exits_with_parent(binary, work):
    """临时服务跟着起它的进程走（客户端文件头「命令行」）：子进程起一份 TestServer、打出服务的进程号之后被 SIGKILL（finally 不跑），
    服务要在 PARENT_DEATH_EXIT_WITHIN_SECONDS 秒内不在；还在就由这一格停掉它（只停这一份服务：它自己一个进程组）、判红。"""
    holder = subprocess.Popen([sys.executable, ENTRY_SCRIPT_PATH, "--selftest-child", "hold-server", binary, os.path.join(work, "held")],
                              stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=cell_environment({}), start_new_session=True)
    server_process, start_ticks, survived, seconds = None, None, False, 0.0
    try:
        ready, _writable, _failed = select.select([holder.stdout], [], [], SERVER_READY_SECONDS)
        line = holder.stdout.readline() if ready else ""
        fields = dict(part.split("=", 1) for part in line.split() if "=" in part)
        if not fields.get("server_pid", "").isdigit():
            return False, f"起服务的子进程 {SERVER_READY_SECONDS} 秒内没打出服务的进程号（读到「{line.strip()}」）"
        server_process = int(fields["server_pid"])
        state = process_state_and_start(server_process)
        start_ticks = state[1] if state else None
        os.killpg(holder.pid, signal.SIGKILL)
        holder.wait(timeout=30)
        killed_at = time.monotonic()
        while start_ticks is not None and still_running(server_process, start_ticks) and time.monotonic() - killed_at < PARENT_DEATH_EXIT_WITHIN_SECONDS:
            time.sleep(0.05)
        seconds = time.monotonic() - killed_at
        survived = start_ticks is not None and still_running(server_process, start_ticks)
        passed = start_ticks is not None and not survived
        return passed, (f"起服务的子进程 {holder.pid} 被 SIGKILL 之后，服务 {server_process} "
                        + (f"过了 {seconds:.1f} 秒还在（要 {PARENT_DEATH_EXIT_WITHIN_SECONDS} 秒内不在；这一格停掉了它）" if survived
                           else f"{seconds:.1f} 秒内不在了" if start_ticks is not None else "打出进程号时已经不在（没起成）"))
    finally:
        if holder.poll() is None:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(holder.pid, signal.SIGKILL)
            holder.wait(timeout=30)
        if server_process is not None and start_ticks is not None and still_running(server_process, start_ticks):
            with contextlib.suppress(ProcessLookupError):
                os.kill(server_process, signal.SIGKILL)   # 点名这一个进程号（刚核过起始时刻还是那一个）：服务是一个进程，不起子进程，不用整组
        holder.stdout.close()
        holder.stderr.close()


def cell_service_problem_judges_given_environment(binary, work):
    """selftest_service_problem 给了环境只判那一份（客户端文件头「命令行」）：本进程没有临时服务、交进去的指着活的那一份 ⇒ 空串；
    本进程指着临时服务、交进去的是空的一份 ⇒ 一句为什么。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        given = {SOCKET_VARIABLE: server.socket, TCP_ADDRESS_VARIABLE: server.tcp_address, TEMPORARY_SERVICE_VARIABLE: server.socket}
        with environment_for({}):
            judged_given = selftest_service_problem(given)
        with environment_for(given):
            judged_empty = selftest_service_problem({})
        passed = judged_given == "" and judged_empty != ""
        return passed, (f"本进程没有临时服务、交进去的指着活的那一份：交「{judged_given or '空串'}」（要空串）；"
                        f"本进程指着临时服务、交进去的是空的一份：交「{judged_empty or '空串'}」（要一句为什么）")
    finally:
        server.stop()


def cell_connection_variables_refused_outside_temporary_service(binary, work):
    """改连接地址与时限的变量只在临时服务底下认（客户端文件头「连哪一处」）：临时服务的标记设成空，只设 socket、设 TCP 地址（socket 指到不在的路径）、
    只设重连总时长、只设等回应时限各读一次，都退 3、点名那个变量、什么都没读；标记指回临时服务再读一次照常读到 absent。
    只设 TCP 不设 socket 那一种不造：开关下会落到本机真服务的 socket。只设时限的两趟把 XDG_RUNTIME_DIR 与多机配置指到不在的地方：
    开关下（照认时限）落到「两处都没有」那一句（同样退 3，但不说「只在临时服务底下认」），不连本机真服务。"""
    server = TestServer(binary, os.path.join(work, "server"))
    notes, passed = [], True
    try:
        reconnect = {RECONNECT_LIMIT_VARIABLE: str(SELFTEST_RECONNECT_SECONDS)}
        nowhere = {"XDG_RUNTIME_DIR": os.path.join(work, "no-runtime"), MULTI_HOST_CONFIGURATION_VARIABLE: os.path.join(work, "no-such.env"),
                   TEMPORARY_SERVICE_VARIABLE: ""}
        for what, changes, variable in (
                ("只设 socket", dict(reconnect, **{SOCKET_VARIABLE: server.socket, TEMPORARY_SERVICE_VARIABLE: ""}), SOCKET_VARIABLE),
                ("设 TCP 地址", dict(reconnect, **{SOCKET_VARIABLE: os.path.join(work, "no-such.sock"), TCP_ADDRESS_VARIABLE: server.tcp_address,
                                                   TEMPORARY_SERVICE_VARIABLE: ""}), TCP_ADDRESS_VARIABLE),
                ("只设重连总时长", dict(nowhere, **reconnect), RECONNECT_LIMIT_VARIABLE),
                ("只设等回应时限", dict(nowhere, **{RESPONSE_TIMEOUT_VARIABLE: "5"}), RESPONSE_TIMEOUT_VARIABLE)):
            code, seconds, output, error = run_child(["read", "override/any"], changes, timeout=SELFTEST_RECONNECT_SECONDS * 8)
            refused = code == EXIT_OTHER and variable in error and "只在临时服务底下认" in error and not output.strip()
            passed = passed and refused
            notes.append(f"{what}、不在临时服务底下：退 {code}（要 3），标准输出「{output.strip()}」（要空），标准错误「{error.strip()[-200:]}」")
        code, seconds, output, error = run_child(["read", "override/any"], dict(unix_environment(server), **reconnect))
        honored = code == 0 and output.strip() == "absent"
        passed = passed and honored
        notes.append(f"标记指回临时服务：退 {code}、标准输出「{output.strip()}」（要 0 与 absent）")
        return passed, "；".join(notes)
    finally:
        server.stop()


def cell_namespace_only_under_temporary_service(binary, work):
    """命名空间只在临时服务底下给（客户端文件头「键空间」命名空间）：标记设着时临时目录下两份库目录交两个不同的「@…」、仓里的库目录同样带；
    标记空着时临时目录下的库目录与仓里的都交空串，LibraryStore 的前缀不带「@」。只调客户端的函数，不起服务（binary 不用）。"""
    first, second = os.path.join(work, "first-root", "singlefs-sample-verdicts"), os.path.join(work, "second-root", "singlefs-sample-verdicts")
    in_repository = os.path.join(REPOSITORY_ROOT, ".git", "singlefs-sample-verdicts")
    marked = {TEMPORARY_SERVICE_VARIABLE: os.path.join(work, "server.sock")}
    unmarked = {}
    under_service = [namespace_of(path, marked) for path in (first, second, in_repository)]
    outside_service = [namespace_of(path, unmarked) for path in (first, in_repository)]
    with environment_for({TEMPORARY_SERVICE_VARIABLE: ""}):
        prefix = LibraryStore(FAMILY_UNIT_VERDICTS, "sample-kind", first).prefix
    passed = (all(value.startswith("@") for value in under_service) and len(set(under_service)) == 3
              and outside_service == ["", ""] and prefix == f"{FAMILY_UNIT_VERDICTS}/sample-kind/".encode())
    return passed, (f"临时服务底下三份库目录交 {under_service}（要三个不同的 @…）；不在临时服务底下交 {outside_service}（要两个空串）；"
                    f"不在临时服务底下临时目录里那份的前缀 {prefix!r}（要不带 @）")


def cell_command_line_read(binary, work):
    """命令行 read（客户端文件头「命令行」）：经 LibraryStore 写一条，起客户端 read 读回：在的原样打出、退 0；不在的退 1、标准输出空；认不得的族退 2。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        directory = os.path.join(work, "read-root", "singlefs-sample-verdicts")
        stored = b'{"verdict": "green"}\n'
        with environment_for(unix_environment(server)):
            LibraryStore(FAMILY_UNIT_VERDICTS, "sample-kind", directory).write("items/present.json", stored)
        outcomes = {}
        for what, arguments in (("在的那一条", (FAMILY_UNIT_VERDICTS, "sample-kind", directory, "items/present.json")),
                                ("不在的那一条", (FAMILY_UNIT_VERDICTS, "sample-kind", directory, "items/absent.json")),
                                ("认不得的族", ("no-such-family", "sample-kind", directory, "items/present.json"))):
            outcomes[what] = run_child([], unix_environment(server), program=[sys.executable, CLIENT_PATH, "read", *arguments])
        present, absent, unknown = outcomes["在的那一条"], outcomes["不在的那一条"], outcomes["认不得的族"]
        passed = (present[0] == 0 and present[2] == stored.decode() and absent[0] == 1 and absent[2] == "" and unknown[0] == EXIT_USAGE)
        return passed, (f"在的那一条：退 {present[0]}、标准输出「{present[2].strip()}」（要 0 与写进去的值）；不在的那一条：退 {absent[0]}、标准输出「{absent[2].strip()}」"
                        f"（要 1 与空）；认不得的族：退 {unknown[0]}（要 2）；标准错误「{(present[3] + absent[3]).strip()[-200:]}」")
    finally:
        server.stop()


def cell_batched_read_answer_over_frame_limit_split(binary, work):
    """回应超过帧长上限就拆键（客户端文件头「容错」）：OVER_LIMIT_VALUE_COUNT 把各正好一段长（SEGMENT_BYTES，不分段存）的值合计超过帧长上限，
    中间夹一条小值与一把不在的键，一次 read_many 读全：逐字节对、次序与问的相同；那几把都没有版本登记键（真没分段，单键回应装得下一帧）。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        generator = random.Random(LARGE_VALUE_SEED + 2)
        sized = [(b"split/sized-%d" % index, generator.randbytes(SEGMENT_BYTES)) for index in range(OVER_LIMIT_VALUE_COUNT)]
        stored = dict(sized)
        stored[b"split/small"] = b"small"
        total = sum(len(value) for _key, value in sized)
        asked = [key for key, _value in sized]
        asked[2:2] = [b"split/small"]
        asked[4:4] = [b"split/missing"]
        expected = [stored.get(key) for key in asked]
        with environment_for(unix_environment(server)):
            client = process_client()
            client.write_many(sized + [(b"split/small", b"small")])
            registry_values = client.read_many_raw([version_registry_key(key) for key, _value in sized])
            try:
                values = client.read_many(asked)
                outcome = "读完"
            except PruneServerFailure as failure:
                values, outcome = None, f"退 {failure.code}（{failure.kind}）"
        unsegmented = registry_values == [None] * len(sized)
        matched = values == expected
        shown = "-" if values is None else [len(value) if value is not None else None for value in values]
        passed = total > MAXIMUM_FRAME_BODY_BYTES and unsegmented and matched
        return passed, (f"{len(sized)} 把各 {SEGMENT_BYTES} 字节的值合计 {total} 字节（帧长上限 {MAXIMUM_FRAME_BODY_BYTES}，要超过）、版本登记键都不在 {unsegmented}；"
                        f"一次读 {len(asked)} 把键：{outcome}，各值长 {shown}，逐字节、次序与问的相同 {matched}（要读完且相同）")
    finally:
        server.stop()


def cell_delete_prefix_by_namespace(binary, work):
    """删前缀（客户端文件头「命令行」delete-prefix）在临时服务底下照命名空间删：两份库目录（命名空间不同）在同一个族同一个种类底下各写几条，
    对其中一份的相对前缀先只数不删（数到的条数对、库不动）、再删（删掉的条数相同）、再数是 0；这一份相对前缀之外的键与另一份的键一条不少。
    命令行 delete-prefix --dry-run 与不带它对另一份各起一次：打 PRUNE_SERVER_DELETE_PREFIX 行，数到与删掉的条数对上，删完那一份空。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        first_directory = os.path.join(work, "first-root", "singlefs-sample-verdicts")
        second_directory = os.path.join(work, "second-root", "singlefs-sample-verdicts")
        with environment_for(unix_environment(server)):
            first = LibraryStore(FAMILY_UNIT_VERDICTS, "sample-kind", first_directory)
            second = LibraryStore(FAMILY_UNIT_VERDICTS, "sample-kind", second_directory)
            first.write_many([("items/a.json", b"1"), ("items/b.json", b"2"), ("items/c.json", b"3"), ("other/d.json", b"4")])
            second.write_many([("items/a.json", b"x"), ("items/b.json", b"y")])
            counted = first.delete_prefix("items/", count_only=True)
            after_counting = [relative for relative, _value in first.scan("")]
            deleted = first.delete_prefix("items/")
            counted_again = first.delete_prefix("items/", count_only=True)
            first_left = [relative for relative, _value in first.scan("")]
            second_left = [relative for relative, _value in second.scan("")]
        outcomes = {}
        for what, extra in (("数", ["--dry-run"]), ("删", [])):
            outcomes[what] = run_child([], unix_environment(server), program=[sys.executable, CLIENT_PATH, "delete-prefix", *extra,
                                                                              FAMILY_UNIT_VERDICTS, "sample-kind", second_directory, "items/"])
        counted_line, deleted_line = outcomes["数"][2], outcomes["删"][2]
        raw_keys = [b"\x1bhex-cell/a", b"\x1bhex-cell/b", b"\x1chex-cell/c", b"\x1dhex-cell/kept"]
        with environment_for(unix_environment(server)):
            second_left_after_command = [relative for relative, _value in second.scan("")]
            process_client().write_many([(key, b"raw") for key in raw_keys])
        for what, extra in (("原样数", ["--dry-run"]), ("原样删", [])):
            outcomes[what] = run_child([], unix_environment(server), program=[sys.executable, CLIENT_PATH, "delete-prefix", *extra, "--hex", "1b", "--hex", "1c"])
        with environment_for(unix_environment(server)):
            raw_left = [key for key, value in zip(raw_keys, process_client().read_many(raw_keys)) if value is not None]
        raw_counted_text, raw_deleted_text = outcomes["原样数"][2], outcomes["原样删"][2]
        raw_passed = (outcomes["原样数"][0] == 0 and outcomes["原样删"][0] == 0 and "PRUNE_SERVER_DELETE_PREFIX_TOTAL mode=dry-run prefixes=2 keys=3" in raw_counted_text
                      and "PRUNE_SERVER_DELETE_PREFIX_TOTAL mode=deleted prefixes=2 keys=3" in raw_deleted_text and raw_left == [raw_keys[3]])
        passed = (raw_passed and counted == 3 and after_counting == ["items/a.json", "items/b.json", "items/c.json", "other/d.json"] and deleted == 3 and counted_again == 0
                  and first_left == ["other/d.json"] and second_left == ["items/a.json", "items/b.json"]
                  and outcomes["数"][0] == 0 and outcomes["删"][0] == 0 and "mode=dry-run" in counted_line and "keys=2" in counted_line
                  and "mode=deleted" in deleted_line and "keys=2" in deleted_line and second_left_after_command == [])
        return passed, (f"这一份：数到 {counted}（要 3）、数完库里 {after_counting}、删掉 {deleted}（要 3）、再数 {counted_again}（要 0）、剩 {first_left}（要只剩 other/d.json）；"
                        f"另一份命名空间的键删之前还有 {second_left}（要两条都在）；命令行：数 退 {outcomes['数'][0]}「{counted_line.strip()}」、删 退 {outcomes['删'][0]}"
                        f"「{deleted_line.strip()}」（要都退 0、行里 keys=2）、删完另一份剩 {second_left_after_command}（要空）；原样键前缀 --hex 1b --hex 1c：数 退 {outcomes['原样数'][0]}、删 退 {outcomes['原样删'][0]}、"
                        f"总数行要是 prefixes=2 keys=3、删完剩 {raw_left}（要只剩 0x1d 起头那一把）；标准错误「{(outcomes['数'][3] + outcomes['删'][3]).strip()[-200:]}」")
    finally:
        server.stop()


def cell_delete_by_item_name_picks_by_value(binary, work):
    """按值挑键删（客户端文件头「命令行」delete-by-item-name）：临时服务底下两个种类各写几条判定记录（JSON，item 字段是项名；一条大到分段存、一条不是 JSON），
    名单点名其中一个种类的两个项名（一个项名有两条记录、一个是分段存的）与另一个种类的一个不存在的项名。命令行 --dry-run：退 0、逐条打键与项名、matched 对、库不动；
    不带 --dry-run：退 0、点名种类里项名命中的几条没了（含分段的），同种类里别的项、不是 JSON 的条目与另一个种类里同名的项一条不少；再 --dry-run 是 matched=0；
    名单里没找到记录的（种类、项名）逐条点名；用法错（缺名单文件）退 2。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        directory = os.path.join(work, "root", "singlefs-sample-verdicts")
        named_kind, other_kind = "sample-stage.sh:replay", "other-stage.sh:replay"
        list_path = os.path.join(work, "names.tsv")
        with open(list_path, "w", encoding="utf-8") as handle:
            handle.write("# 种类\t项名\n" + f"{named_kind}\tsample-sparrow\n{named_kind}\tsample-heron\n{other_kind}\tsample-nobody\n")
        large_record = json.dumps({"verdict": "green", "item": "sample-heron", "padding": "x" * (SEGMENT_BYTES + 10)}).encode()

        def record(item):
            return json.dumps({"verdict": "green", "item": item}).encode()

        with environment_for(unix_environment(server)):
            named = LibraryStore(FAMILY_UNIT_VERDICTS, named_kind, directory)
            other = LibraryStore(FAMILY_UNIT_VERDICTS, other_kind, directory)
            named.write_many([("keys/aaa.json", record("sample-sparrow")), ("keys/bbb.json", record("sample-sparrow")), ("keys/ccc.json", record("real-lighthouse")),
                              ("keys/fff.json", large_record), ("items/xyz", b"not-json")])
            other.write_many([("keys/ddd.json", record("sample-sparrow")), ("keys/eee.json", record("real-harbor"))])
            before = (sorted(relative for relative, _value in named.scan("")), sorted(relative for relative, _value in other.scan("")))
        environment = unix_environment(server)
        program = [sys.executable, CLIENT_PATH, "delete-by-item-name"]
        counted = run_child([], environment, program=[*program, "--dry-run", FAMILY_UNIT_VERDICTS, directory, list_path])
        with environment_for(unix_environment(server)):
            after_counting = (sorted(relative for relative, _value in named.scan("")), sorted(relative for relative, _value in other.scan("")))
        deleted = run_child([], environment, program=[*program, FAMILY_UNIT_VERDICTS, directory, list_path])
        with environment_for(unix_environment(server)):
            named_left = sorted(relative for relative, _value in named.scan(""))
            other_left = sorted(relative for relative, _value in other.scan(""))
            segments_left = [key for key, _value in process_client().raw_scan_stream(named.key("keys/fff.json"))]
        counted_again = run_child([], environment, program=[*program, "--dry-run", FAMILY_UNIT_VERDICTS, directory, list_path])
        misuse = run_child([], environment, program=[*program, FAMILY_UNIT_VERDICTS, directory])
        counted_text, deleted_text, again_text = counted[2], deleted[2], counted_again[2]
        named_prefix = named.prefix.decode()
        expected_rows = [f"kind={named_kind} keys=1 key={named_prefix}keys/aaa.json item=sample-sparrow", f"kind={named_kind} keys=1 key={named_prefix}keys/bbb.json item=sample-sparrow"]
        passed = (counted[0] == 0 and deleted[0] == 0 and counted_again[0] == 0 and misuse[0] == EXIT_USAGE
                  and all(row in counted_text and row in deleted_text for row in expected_rows)
                  and f"kind={named_kind} keys=" in counted_text and f"key={named_prefix}keys/fff.json item=sample-heron" in counted_text
                  and "mode=dry-run kinds=2 matched=3 " in counted_text and "mode=deleted kinds=2 matched=3 " in deleted_text and "names_not_found=1" in counted_text
                  and f"PRUNE_SERVER_DELETE_BY_ITEM_NAME_NOT_FOUND kind={other_kind} item=sample-nobody" in counted_text
                  and "mode=dry-run kinds=2 matched=0 " in again_text
                  and after_counting == before
                  and named_left == ["items/xyz", "keys/ccc.json"] and other_left == ["keys/ddd.json", "keys/eee.json"] and segments_left == [])
        return passed, (f"命令行 --dry-run 退 {counted[0]}「{counted_text.strip()[-500:]}」、数完库里 {after_counting}（要和写入后的 {before} 一样）；"
                        f"删 退 {deleted[0]}「{deleted_text.strip()[-300:]}」、点名种类剩 {named_left}（要 items/xyz、keys/ccc.json）、另一个种类剩 {other_left}（要两条都在）、"
                        f"分段存的那条的原键与段键剩 {segments_left}（要空）；再数 退 {counted_again[0]}「{again_text.strip()[-200:]}」（要 matched=0）；"
                        f"缺名单文件退 {misuse[0]}（要 {EXIT_USAGE}）；标准错误「{(counted[3] + deleted[3]).strip()[-200:]}」")
    finally:
        server.stop()


CONTINUED_PREFIX = b"continued/"
CONTINUED_VALUE_BYTES = 12 * 1024 * 1024   # prefix-scan-continues-after-the-last-key：一把的字节数，不到一段长（不分段存）
CONTINUED_VALUE_COUNT = 6                  # 同一格：几把，合计超过帧长上限（一帧装不下全部）


def cell_prefix_scan_continues_after_the_last_key(binary, work):
    """按前缀扫一帧装不下全部（客户端文件头「键空间」raw_scan_stream）：前缀下 CONTINUED_VALUE_COUNT 把各 CONTINUED_VALUE_BYTES 字节的值加两条小值、
    前缀前后各一条别的；raw_scan_stream 与 scan 交的条目逐把相同（比 sha256）、按键升序、不重不漏，发的扫帧数不多于条目数加一，第二帧起带续扫起点。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        generator = random.Random(LARGE_VALUE_SEED + 8)
        stored = [(CONTINUED_PREFIX + b"big-%d" % index, generator.randbytes(CONTINUED_VALUE_BYTES)) for index in range(CONTINUED_VALUE_COUNT)]
        stored += [(CONTINUED_PREFIX + b"small-a", b"a"), (CONTINUED_PREFIX + b"small-b", b"b")]
        outside = [(b"continue", b"before-the-prefix"), (b"continued0", b"after-the-prefix")]
        expected = sorted((key, hashlib.sha256(value).hexdigest()) for key, value in stored)
        total = sum(len(value) for _key, value in stored)
        with environment_for(unix_environment(server)):
            client = process_client()
            client.write_many(stored + outside)
            del stored
            asked = []
            unwrapped = client.raw_scan

            def counted_raw_scan(prefix, after_key=b""):
                asked.append(after_key)
                return unwrapped(prefix, after_key)

            client.raw_scan = counted_raw_scan
            raw = [(key, hashlib.sha256(value).hexdigest()) for key, value in client.raw_scan_stream(CONTINUED_PREFIX)]
            raw_frames, resumed = len(asked), sum(1 for after_key in asked if after_key)
            asked.clear()
            scanned = [(key, hashlib.sha256(value).hexdigest()) for key, value in client.scan_stream(CONTINUED_PREFIX)]
            scan_frames = len(asked)
        raw_ok, scan_ok = raw == expected, scanned == expected
        bounded = 2 <= raw_frames <= len(expected) + 1 and scan_frames <= len(expected) + 1 and resumed == raw_frames - 1
        passed = total > MAXIMUM_FRAME_BODY_BYTES and raw_ok and scan_ok and bounded
        return passed, (f"前缀下 {len(expected)} 条、合计 {total} 字节（帧长上限 {MAXIMUM_FRAME_BODY_BYTES}，要超过）；raw_scan_stream 交 {len(raw)} 条、"
                        f"逐把相同且按键升序不重不漏 {raw_ok}，发了 {raw_frames} 帧扫（其中带续扫起点的 {resumed} 帧）；scan 交 {len(scanned)} 条、相同 {scan_ok}、"
                        f"发了 {scan_frames} 帧（帧数要在 2 到条数加一 {len(expected) + 1} 之间，除头一帧都带续扫起点）")
    finally:
        server.stop()


STORAGE_FAILURE_LINE_SECONDS = 20     # write-storage-failure-replayed：等服务的故障日志出现写失败那一行的上限
STORAGE_REPLAY_RECONNECT_SECONDS = 60  # 同一格：删掉造错文件之前写一直在重发，总时长上限给宽
STORAGE_FAILING_RECONNECT_SECONDS = 2  # 同一格：造错文件一直在时的重连总时长上限
STORAGE_FAILING_GRACE_SECONDS = 8      # 同一格：退 75 要在总时长上限加这么多秒里


def cell_write_storage_failure_replayed(binary, work):
    """写帧回存储出错照同一请求号重放（客户端文件头「容错」写帧回存储出错）：服务带 --selftest-write-failure-file 起（造错文件在时每组写批都当写失败、服务不退）。
    造错文件在时写一条，等故障日志出现 storage-write-failed 之后删掉造错文件：这一次写做完、重发过、读回值；
    造错文件一直在、重连总时长 STORAGE_FAILING_RECONNECT_SECONDS 秒：写在限时里退 75（storage-failing），不退 3。"""
    failure_file = os.path.join(work, "write-failure")
    server = TestServer(binary, os.path.join(work, "server"), extra=["--selftest-write-failure-file", failure_file])
    faults = os.path.join(server.directory, "faults.log")
    try:
        with open(failure_file, "w", encoding="utf-8"):
            pass
        outcome = {}
        with environment_for(dict(unix_environment(server), **{RECONNECT_LIMIT_VARIABLE: str(STORAGE_REPLAY_RECONNECT_SECONDS)})):
            client = process_client()

            def write():
                try:
                    client.write_many([(b"storage/replayed", b"value")])
                    outcome["write"] = "做完"
                except PruneServerFailure as failure:
                    outcome["write"] = f"退 {failure.code}（{failure.kind}）：{failure.message[-200:]}"

            writer = threading.Thread(target=write)
            writer.start()
            deadline, seen = time.monotonic() + STORAGE_FAILURE_LINE_SECONDS, False
            while not seen and writer.is_alive() and time.monotonic() < deadline:
                with contextlib.suppress(OSError):
                    with open(faults, encoding="utf-8", errors="replace") as handle:
                        seen = "kind=storage-write-failed" in handle.read()
                time.sleep(0.05)
            os.remove(failure_file)
            writer.join(timeout=CHILD_TIMEOUT_SECONDS)
            finished = not writer.is_alive()
            replays = client.storage_failures
            value = client.read_many([b"storage/replayed"])[0] if finished else None
        with open(failure_file, "w", encoding="utf-8"):
            pass
        with environment_for(dict(unix_environment(server), **{RECONNECT_LIMIT_VARIABLE: str(STORAGE_FAILING_RECONNECT_SECONDS)})):
            started = time.monotonic()
            try:
                process_client().write_many([(b"storage/failing", b"value")])
                failing, failing_kind = "做完", None
            except PruneServerFailure as failure:
                failing, failing_kind = f"退 {failure.code}（{failure.kind}）", (failure.code, failure.kind)
            failing_seconds = time.monotonic() - started
        os.remove(failure_file)
        replay_ok = seen and finished and outcome.get("write") == "做完" and replays >= 1 and value == b"value"
        failing_ok = failing_kind == (EXIT_UNAVAILABLE, "storage-failing") and failing_seconds < STORAGE_FAILING_RECONNECT_SECONDS + STORAGE_FAILING_GRACE_SECONDS
        passed = replay_ok and failing_ok
        return passed, (f"造错文件在时写：故障日志见到写失败 {seen}、写 {outcome.get('write', '没回来')}、照同一请求号重发 {replays} 次、读回 {value!r}"
                        f"（要做完、重发至少 1 次、读回 b'value'）；造错文件一直在：{failing}（{failing_seconds:.1f} 秒，要 {STORAGE_FAILING_RECONNECT_SECONDS + STORAGE_FAILING_GRACE_SECONDS} 秒内退 75（storage-failing））")
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(failure_file)
        server.stop()


GPU_SELFTEST_STREAM_KEY = b"\x1cpython-gpu-selftest-stream"   # gpu-pending-rows-written-only-if-absent、gpu-conclusion-write-guarded：流键（以表标签那样的字节起头）
GPU_SELFTEST_SERVICE_START_RANDOM = 0x5E1F7E57                   # 同两格：持有者身份里的服务进程随机号
GPU_SELFTEST_LEASE_MILLISECONDS = 60_000                         # gpu-conclusion-write-guarded：认领的租期，比一格跑的时间长得多
GPU_PINNED_PENDING_ROW_HEXADECIMAL = ("02000000047202000000abcd0100000000000000020000000000000001000000670100000076010300000000000000")
# 同上：与 prune-server/src/gpu_judging_rows.rs 单测 gpu_keys_holder_identity_and_pending_row_match_the_bytes_pinned_for_the_python_client 同一组字面值


def gpu_sample_pending_row(block_number, recording_table_key):
    return encode_gpu_pending_row(recording_table_key, hashlib.sha256(b"selftest judge table").digest(), block_number * 65536, (block_number + 1) * 65536,
                                  b"selftest-gpu-judge-group", b"selftest-conclusion-version", False, 1)


def cell_gpu_pending_rows_written_only_if_absent(binary, work):
    """待判行不在才写（客户端文件头「GPU 判的三族」）：录两块的那一帧写 2 条，重录那一帧（块 7 已在、块 9 新的）写 1 条，块 7 不改，按流的前缀扫回三行、按块号升序；
    键、持有者身份、待判行的值与 Rust 那一份钉住的字节相同。判别力：弄坏开关 write-if-absent-sent-as-write。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        pinned_row = encode_gpu_pending_row(b"\x04r", b"\xab\xcd", 1, 2, b"g", b"v", True, 3)
        pinned = (gpu_pending_key(b"\x00/", 42) == b"gpu-pending/002f/000000000000002a"
                  and gpu_claim_holder_identity(0x1234, 1, 2) == b"gpu-service/0000000000001234/card-1/thread-2"
                  and pinned_row == bytes.fromhex(GPU_PINNED_PENDING_ROW_HEXADECIMAL)
                  and decode_gpu_pending_row(pinned_row)["pool_sequence"] == 3)
        with environment_for(unix_environment(server)):
            client = process_client()
            stream = GPU_SELFTEST_STREAM_KEY
            recorded = [(gpu_pending_key(stream, number), gpu_sample_pending_row(number, b"\x04recorded")) for number in (7, 8)]
            again = [(gpu_pending_key(stream, number), gpu_sample_pending_row(number, b"\x04again")) for number in (7, 9)]
            written_recorded, _ = client.write_if_absent_frame(recorded)
            written_again, _ = client.write_if_absent_frame(again)
            stored = client.read_many_raw([gpu_pending_key(stream, 7), gpu_pending_key(stream, 9)])
            scanned = [key for key, _value in client.raw_scan_stream(gpu_pending_prefix_of_stream(stream))]
        expected_keys = [gpu_pending_key(stream, number) for number in (7, 8, 9)]
        kept = stored == [recorded[0][1], again[1][1]]
        passed = pinned and written_recorded == 2 and written_again == 1 and kept and scanned == expected_keys
        return passed, (f"字面值钉住的键、持有者身份、待判行的值对得上 {pinned}；录两块的那一帧写 {written_recorded} 条（要 2）、重录那一帧写 {written_again} 条（要 1），"
                        f"块 7 不改、块 9 是重录那一帧写的 {kept}，按流的前缀扫回 {len(scanned)} 行、按块号升序 {scanned == expected_keys}")
    finally:
        server.stop()


def cell_gpu_conclusion_write_guarded(binary, work):
    """写结论核认领与守卫那一行（客户端文件头「GPU 判的三族」）：别的持有者持着、租期没到交 claim-held-by-other；持有者自己写（强制现判）交 written 3 条；
    读到「不在」的旧写方再写交 guard-row-changed、带库里那一份的 SHA-256；库里结论块是持有者写的、待判行删了、强制现判行对。
    判别力：弄坏开关 guard-refusal-exits、gpu-rejudged-row-dropped。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        stream, block, pool_sequence = GPU_SELFTEST_STREAM_KEY, 1, 4
        holder = gpu_claim_holder_identity(GPU_SELFTEST_SERVICE_START_RANDOM, 0, 0)
        other = gpu_claim_holder_identity(GPU_SELFTEST_SERVICE_START_RANDOM, 1, 0)
        conclusion_key, side = b"\x10python-gpu-conclusion/0001", [(b"\x11python-gpu-side/0001", b"side row")]
        with environment_for(unix_environment(server)):
            client = process_client()
            try:
                pending_row = gpu_sample_pending_row(block, b"\x04recording")
                client.write_if_absent_frame([(gpu_pending_key(stream, block), pending_row)])
                claimed = client.claim(holder, [gpu_claim_key(stream, block)], GPU_SELFTEST_LEASE_MILLISECONDS)
                by_other = client.write_guarded_conclusion(**gpu_conclusion_write(stream, block, other, conclusion_key, None, side, b"verdicts of the other", None,
                                                                                  pending_row))
                by_holder = client.write_guarded_conclusion(**gpu_conclusion_write(stream, block, holder, conclusion_key, None, side, b"verdicts of the holder",
                                                                                   pool_sequence, pending_row))
                stale = client.write_guarded_conclusion(**gpu_conclusion_write(stream, block, other, conclusion_key, None, [], b"verdicts of the other", None,
                                                                                pending_row))
                stored = client.read_many_raw([conclusion_key, gpu_pending_key(stream, block), gpu_rejudged_key(pool_sequence, stream, block)])
            except PruneServerFailure as failure:
                return False, f"写结论那几步退出了：退 {failure.code}（{failure.kind}）：{failure.message[-200:]}（守卫不过该交给调用方、不退出）"
        holder_digest = guard_row_state_of(b"verdicts of the holder")
        refused_for_holder = by_other.get("result") == "claim-held-by-other" and by_other.get("holder") == holder
        written_by_holder = by_holder == {"result": "written", "entries_written": 3, "already_applied": False}
        stale_refused = stale.get("result") == "guard-row-changed" and stale.get("current") == holder_digest
        rejudged = decode_gpu_rejudged_row(stored[2]) if stored[2] is not None else None
        stored_ok = stored[0] == b"verdicts of the holder" and stored[1] is None and rejudged == (holder, holder_digest)
        passed = claimed == [("claimed", None, None)] and refused_for_holder and written_by_holder and stale_refused and stored_ok
        return passed, (f"认领 {claimed}；别的持有者写交 {by_other.get('result')}（要 claim-held-by-other、持有者对 {by_other.get('holder') == holder}）；"
                        f"持有者写交 {by_holder}（要 written 3 条）；旧写方再写交 {stale.get('result')}（要 guard-row-changed、带库里那一份的 SHA-256 {stale.get('current') == holder_digest}）；"
                        f"结论块是持有者写的、待判行删了、强制现判行对 {stored_ok}")
    finally:
        server.stop()


GPU_LISTING_SHORT_LEASE_MILLISECONDS = 300      # gpu-claim-listing-lists-live-claims：会过期的那一块认领的短租期
GPU_LISTING_WAIT_PAST_SHORT_LEASE_SECONDS = 1.0  # 同一格：认领之后等这么久，短租期一定过了（服务按自己的时钟判）
GPU_CLAIM_KEY_BLOCK_NUMBER_DIGITS = 16           # 同一格：认领键末尾的块号位数（客户端文件头「键空间」：16 位小写十六进制）


def cell_gpu_claim_listing_lists_live_claims(binary, work):
    """列认领（客户端文件头「GPU 判的三族」）：持有者甲认领主流的头两块与另一条流的块（长租期），持有者乙认领主流的块 3（短租期、等过期）：
    按族前缀列只回甲的三块（键升序、持有者身份对、剩余租期在 1 到长租期之间），乙那块过了租期不列；按主流的认领键前缀列只回主流的头两块；
    一帧至多两条接着列：头一帧两条且截断，下一帧一条不截断，合起来与全列相同。判别力：弄坏开关 claim-listing-prefix-widened、claim-listing-resume-key-dropped。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        stream, other_stream = GPU_SELFTEST_STREAM_KEY, GPU_SELFTEST_STREAM_KEY + b"-other"
        live = gpu_claim_holder_identity(GPU_SELFTEST_SERVICE_START_RANDOM, 0, 0)
        expiring = gpu_claim_holder_identity(GPU_SELFTEST_SERVICE_START_RANDOM, 1, 0)
        live_keys = [gpu_claim_key(stream, 1), gpu_claim_key(stream, 2), gpu_claim_key(other_stream, 1)]
        with environment_for(unix_environment(server)):
            client = process_client()
            try:
                claimed_live = client.claim(live, live_keys, GPU_SELFTEST_LEASE_MILLISECONDS)
                claimed_expiring = client.claim(expiring, [gpu_claim_key(stream, 3)], GPU_LISTING_SHORT_LEASE_MILLISECONDS)
                time.sleep(GPU_LISTING_WAIT_PAST_SHORT_LEASE_SECONDS)
                everything = client.list_claims(GPU_CLAIM_FAMILY_PREFIX)
                one_stream = client.list_claims(gpu_claim_key(stream, 0)[:-GPU_CLAIM_KEY_BLOCK_NUMBER_DIGITS])
                page_of_two, page_of_two_truncated = client.list_claims_after(GPU_CLAIM_FAMILY_PREFIX, b"", 2)
                page_after_the_two, page_after_the_two_truncated = client.list_claims_after(GPU_CLAIM_FAMILY_PREFIX, page_of_two[-1][0] if page_of_two else b"", 2)
            except PruneServerFailure as failure:
                return False, f"列认领那几步退出了：退 {failure.code}（{failure.kind}）：{failure.message[-200:]}"
        everything_ok = ([key for key, _holder, _remaining in everything] == sorted(live_keys)
                         and all(holder == live and 0 < remaining <= GPU_SELFTEST_LEASE_MILLISECONDS for _key, holder, remaining in everything))
        one_stream_ok = [key for key, _holder, _remaining in one_stream] == live_keys[:2]
        paging_ok = (len(page_of_two) == 2 and page_of_two_truncated and len(page_after_the_two) == 1 and not page_after_the_two_truncated
                     and [key for key, _holder, _remaining in page_of_two + page_after_the_two] == [key for key, _holder, _remaining in everything])
        passed = claimed_live == [("claimed", None, None)] * 3 and claimed_expiring == [("claimed", None, None)] and everything_ok and one_stream_ok and paging_ok
        return passed, (f"认领 {claimed_live}、{claimed_expiring}；按族前缀列回 {len(everything)} 条（要甲的三块、升序、持有者与剩余租期对，过期的乙那块不列）{everything_ok}；"
                        f"按主流的前缀列回 {len(one_stream)} 条（要它的头两块）{one_stream_ok}；一帧至多两条：头一帧 {len(page_of_two)} 条、截断 {page_of_two_truncated}，"
                        f"下一帧 {len(page_after_the_two)} 条、截断 {page_after_the_two_truncated}，合起来与全列相同 {paging_ok}")
    finally:
        server.stop()


CONCLUSION_SAMPLE_BLOCK_KEY_BYTES = 72     # delete-conclusions-* 各格：假验证包源码文件头写的块键长度，样本键照它造
CONCLUSION_SAMPLE_VERSION_BYTES = 32       # 同：键里结论种类字节之后的版本
CONCLUSION_SAMPLE_TAGS = (0x05, 0x08, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x11, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1D, 0x21)   # 格里写过键的表标签
CONCLUSION_SAMPLE_LARGE_VALUE_BYTES = 16 * 1024 * 1024 + 10   # 一行大到分段存（一段是 FRAME_BATCH_BYTES = 16 MiB）
CONCLUSION_SAMPLE_SHORT_LEASE_MILLISECONDS = 300
CONCLUSION_SAMPLE_WAIT_PAST_SHORT_LEASE_SECONDS = 1.0
# 假验证包源码里 key_tag 的写法（成员名、表标签）：与客户端抄的、验证包文件头与代码写的是同一份事实，这一格的预期照它写，不 import 客户端的常量
CONCLUSION_SAMPLE_MEMBER_TAGS = (("ConclusionBlocks", "0A"), ("ConclusionViolations", "0B"), ("RecoveryWords", "0C"), ("ConclusionCountVectors", "0D"),
                                 ("ConclusionBlockAttributes", "0E"), ("AbandonedRootsNamedByThePoolChecker", "18"), ("ObservedImageJudgements", "17"),
                                 ("PackedTablesVerdictWords", "21"), ("ObservedImages", "15"), ("ObserverBlocks", "16"), ("ObserverReadSetWitnessRows", "1A"),
                                 ("ObserverReadSets", "1B"), ("VerifierVerdicts", "08"), ("PairingConclusions", "19"))
# 每个结论种类 --dry-run 该打的每一片：(表标签, 成员名, 选法, 数到几条)；键里带结论种类的表数的是行，整张表数的是键（21 写了两行、别的各一行）
CONCLUSION_SAMPLE_EXPECTED_SLICES = {
    "image-judgement": ((0x0A, "ConclusionBlocks", "kind-byte:01", 2), (0x0B, "ConclusionViolations", "kind-byte:01", 1), (0x0C, "RecoveryWords", "kind-byte:01", 0),
                        (0x0D, "ConclusionCountVectors", "kind-byte:01", 0), (0x0E, "ConclusionBlockAttributes", "kind-byte:01", 0),
                        (0x18, "AbandonedRootsNamedByThePoolChecker", "kind-byte:01", 1), (0x17, "ObservedImageJudgements", "whole-table", 1),
                        (0x21, "PackedTablesVerdictWords", "whole-table", 2)),
    "observer": ((0x0A, "ConclusionBlocks", "kind-byte:02", 1), (0x0B, "ConclusionViolations", "kind-byte:02", 1), (0x0C, "RecoveryWords", "kind-byte:02", 0),
                 (0x0D, "ConclusionCountVectors", "kind-byte:02", 0), (0x0E, "ConclusionBlockAttributes", "kind-byte:02", 0),
                 (0x18, "AbandonedRootsNamedByThePoolChecker", "kind-byte:02", 1), (0x15, "ObservedImages", "whole-table", 1), (0x16, "ObserverBlocks", "whole-table", 1),
                 (0x1A, "ObserverReadSetWitnessRows", "whole-table", 1), (0x1B, "ObserverReadSets", "whole-table", 1)),
    "facts-verifier": ((0x08, "VerifierVerdicts", "whole-table", 1),),
    "pairing": ((0x19, "PairingConclusions", "whole-table", 1),),
}


def conclusion_sample_key(tag, number, kind_byte=None, tail=b""):
    """样本键：键里带结论种类的表是「表标签 ‖ 块键（number 重复 72 次）‖ 结论种类 ‖ 版本（32 个 number）‖ tail」，kind_byte 是 None 的整张表是「表标签 ‖ number 重复 32 次」。"""
    if kind_byte is None:
        return bytes([tag]) + bytes([number]) * CONCLUSION_SAMPLE_VERSION_BYTES
    return bytes([tag]) + bytes([number]) * CONCLUSION_SAMPLE_BLOCK_KEY_BYTES + bytes([kind_byte]) + bytes([number]) * CONCLUSION_SAMPLE_VERSION_BYTES + tail


def conclusion_sample_rows():
    """样本行：{标签: 键}，标签是 (组, 表标签, 号)。组 image-judgement、observer 是键里带结论种类的行（表 0A、0B、18 各一行，两种各一份，0A 的 image-judgement 另有一行大值），
    组 whole 是整张删的表与不属于任何结论种类的表（05、0F、11、1D 当旁观者）。"""
    rows = {}
    for group, kind_byte in (("image-judgement", 0x01), ("observer", 0x02)):
        for tag in (0x0A, 0x0B, 0x18):
            rows[(group, tag, 1)] = conclusion_sample_key(tag, 1, kind_byte, tail=b"\x00" * 7 + b"\x05" if tag == 0x0B else b"")
    rows[("image-judgement", 0x0A, 2)] = conclusion_sample_key(0x0A, 2, 0x01)
    for tag, count in ((0x17, 1), (0x21, 2), (0x15, 1), (0x16, 1), (0x1A, 1), (0x1B, 1), (0x08, 1), (0x19, 1), (0x05, 1), (0x0F, 1), (0x11, 1), (0x1D, 1)):
        for number in range(1, count + 1):
            rows[("whole", tag, number)] = conclusion_sample_key(tag, number)
    return rows


def write_conclusion_sample_rows(client, rows):
    """样本行写进临时服务（一行大值，别的行值是一个字节）：返回大值那一行的键。"""
    large_key = rows[("image-judgement", 0x0A, 2)]
    client.write_many([(key, b"x" if key != large_key else b"L" * CONCLUSION_SAMPLE_LARGE_VALUE_BYTES) for key in rows.values()])
    return large_key


def stored_conclusion_keys(client):
    """样本用到的各表标签底下服务里原样的全部键（段键与版本登记键也在），升序。"""
    return sorted(key for tag in CONCLUSION_SAMPLE_TAGS for key, _value in client.raw_scan_stream(bytes([tag])))


def conclusion_source_tree(root, block_key_bytes=CONCLUSION_SAMPLE_BLOCK_KEY_BYTES, blocks_tag="0A", with_store_file=True):
    """假验证包源码树（root 底下 crates/singlefs-verification/src/…）：只写 delete-conclusions 对账要读的几处——verdict_store.rs 的文件头（键里带结论种类的六张表的行、
    「结论种类 = 01（镜像判定…）、02（观察者）」、「定长 N 字节」）与 key_tag 的各行。
    blocks_tag 只改 key_tag 里 ConclusionBlocks 那一行的标签（客户端要删的是 0A）；with_store_file 为假不写 verdict_store.rs。"""
    source = os.path.join(root, "crates", "singlefs-verification", "src")
    os.makedirs(source, exist_ok=True)
    header = (f"//! 块键 = 段写表与起步镜像的摘要 ‖ 枚举计划哈希 ‖ 块起点，定长 {block_key_bytes} 字节。\n"
              "//! | 表 | 键 | 值 |\n"
              "//! | 结论块 | `0A` ‖ 块键 ‖ 结论种类 ‖ 版本（32 字节） | 每个状态一个判定向量编号 |\n"
              "//! | 结论违例 | `0B` ‖ 块键 ‖ 结论种类 ‖ 版本 ‖ 块内序号（8 字节） | 判定向量编号 |\n"
              "//! | 恢复字 | `0C` ‖ 块键 ‖ 结论种类 ‖ 版本 | 每个状态的字 |\n"
              "//! | 计数向量 | `0D` ‖ 块键 ‖ 结论种类 ‖ 版本 | 计数 |\n"
              "//! | 块属性 | `0E` ‖ 块键 ‖ 结论种类 ‖ 版本 | 在哪判的 |\n"
              "//! | 点名的被抛弃根 | `18` ‖ 块键 ‖ 结论种类 ‖ 版本 | 被抛弃根 |\n"
              "//! 结论种类 = `01`（镜像判定：GPU 判器与 CPU 判器判出的都是这一种）、`02`（观察者），别的字节读回认不出。\n")
    tags = "".join(f"            Self::{member} => 0x{blocks_tag if member == 'ConclusionBlocks' else tag},\n" for member, tag in CONCLUSION_SAMPLE_MEMBER_TAGS)
    if with_store_file:
        with open(os.path.join(source, "verdict_store.rs"), "w", encoding="utf-8") as handle:
            handle.write(header + "impl VerdictStoreTable {\n    fn key_tag(self) -> u8 {\n        match self {\n" + tags + "        }\n    }\n}\n")
    return root


def conclusions_command(server, *arguments):
    """起命令行 delete-conclusions：交 (退出码, 用时, 标准输出, 标准错误)。"""
    return run_child([], unix_environment(server), program=[sys.executable, CLIENT_PATH, "delete-conclusions", *arguments])


def slice_lines_of(output):
    """命令行打出的每一片：[(表标签, 成员名, 选法, 数到几条)]，次序照打的次序。"""
    slices = []
    for line in output.splitlines():
        if line.startswith("PRUNE_SERVER_DELETE_CONCLUSIONS_SLICE "):
            words = dict(word.split("=", 1) for word in line.split()[1:])
            slices.append((int(words["table"], 16), words["member"], words["select"], int(words["keys"])))
    return slices


def keys_of_the_rows(rows, groups_and_whole_tags):
    """rows 里属于 groups_and_whole_tags（组名，或 whole 组里的表标签）的样本键。"""
    return {key for (group, tag, _number), key in rows.items() if group in groups_and_whole_tags or (group == "whole" and tag in groups_and_whole_tags)}


def keys_left_without(stored, deleted_rows):
    """stored 里去掉 deleted_rows 这几把原键与它们的段键、版本登记键（都以原键起头）之后剩下的。"""
    return [key for key in stored if not any(key.startswith(row) for row in deleted_rows)]


def cell_delete_conclusions_by_kind(binary, work):
    """崩溃判定库按结论种类整族删（客户端文件头「命令行」delete-conclusions）：临时服务底下照验证包文件头的形状写样本行（键里带结论种类的 0A、0B、18 两种结论种类各几行，
    0A 的 image-judgement 里有一行大到分段存；整张删的 17、21、15、16、1A、1B、08、19；旁观者 05、0F、11、1D），--repository-root 指一棵与样本一致的假源码树。
    image-judgement --dry-run：退 0、每一片数到的条数对、库一行不动；删：退 0、结论种类 01 的行（含大值的段键与版本登记键）与 17、21 没了，
    结论种类 02 的行、observer 的整张表、08、19、旁观者一条不少；再 --dry-run 每一片数到 0。observer、facts-verifier、pairing 各删一遍：每一次只少各自那几片。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        tree = conclusion_source_tree(os.path.join(work, "tree"))
        rows = conclusion_sample_rows()
        with environment_for(unix_environment(server)):
            client = process_client()
            large_key = write_conclusion_sample_rows(client, rows)
            before = stored_conclusion_keys(client)
        segments_written = [key for key in before if key.startswith(large_key) and key != large_key]
        problems = [] if segments_written else ["样本里那一行大值没有分段存（服务里没有它的段键）"]
        remaining = list(before)
        groups = (("image-judgement", {"image-judgement", 0x17, 0x21}), ("observer", {"observer", 0x15, 0x16, 0x1A, 0x1B}), ("facts-verifier", {0x08}), ("pairing", {0x19}))
        for name, group in groups:
            counted = conclusions_command(server, "--dry-run", "--repository-root", tree, name)
            with environment_for(unix_environment(server)):
                after_counting = stored_conclusion_keys(process_client())
            expected_slices = list(CONCLUSION_SAMPLE_EXPECTED_SLICES[name])
            if counted[0] != 0 or slice_lines_of(counted[2]) != expected_slices:
                problems.append(f"{name} --dry-run 退 {counted[0]}、打的各片是 {slice_lines_of(counted[2])}（要 {expected_slices}）：{counted[3].strip()[-200:]}")
            if after_counting != remaining:
                problems.append(f"{name} --dry-run 改了库：键 {len(after_counting)} 把（要 {len(remaining)} 把）")
            deleted = conclusions_command(server, "--repository-root", tree, name)
            remaining = keys_left_without(remaining, keys_of_the_rows(rows, group))
            with environment_for(unix_environment(server)):
                after_deleting = stored_conclusion_keys(process_client())
            if deleted[0] != 0 or f"PRUNE_SERVER_DELETE_CONCLUSIONS mode=deleted kind={name} " not in deleted[2]:
                problems.append(f"{name} 删 退 {deleted[0]}：{deleted[2].strip()[-300:]} {deleted[3].strip()[-200:]}")
            if after_deleting != remaining:
                gone = sorted(set(remaining) - set(after_deleting))
                extra = sorted(set(after_deleting) - set(remaining))
                problems.append(f"{name} 删完库里不对：多删了 {len(gone)} 把、该删的还剩 {len(extra)} 把")
            again = conclusions_command(server, "--dry-run", "--repository-root", tree, name)
            if again[0] != 0 or any(keys for _tag, _member, _select, keys in slice_lines_of(again[2])):
                problems.append(f"{name} 删完再 --dry-run 该每一片 0 条：{slice_lines_of(again[2])}")
        return not problems, ("；".join(problems) if problems else
                              f"四个结论种类各 --dry-run（库不动）、删（只少各自的行与整张表，含大值的 {len(segments_written)} 把段键）、再数到 0，旁观者与别种类的行一条不少")
    finally:
        server.stop()


def cell_delete_conclusions_refuses_while_claims_are_live(binary, work):
    """有持有者拒绝删（客户端文件头「命令行」delete-conclusions）：一个持有者认领了 gpu-claim/ 下一块、租期长：删退 1、点名这个持有者、库一行不少；
    --dry-run 退 0、照列持有者、库不动。另一份服务里一个持有者认领了一块、租期短、等过期之后：删退 0、删了（租期过了的不算持有者）。
    结论种类不认得、没给、给两个都退 2、库不动。"""
    problems = []
    for lease_milliseconds, expires in ((GPU_SELFTEST_LEASE_MILLISECONDS, False), (CONCLUSION_SAMPLE_SHORT_LEASE_MILLISECONDS, True)):
        server = TestServer(binary, os.path.join(work, f"server-{'short' if expires else 'long'}"))
        try:
            tree = conclusion_source_tree(os.path.join(work, "tree"))
            rows = conclusion_sample_rows()
            holder = gpu_claim_holder_identity(GPU_SELFTEST_SERVICE_START_RANDOM, 0, 0)
            with environment_for(unix_environment(server)):
                client = process_client()
                write_conclusion_sample_rows(client, rows)
                claimed = client.claim(holder, [gpu_claim_key(GPU_SELFTEST_STREAM_KEY, 1)], lease_milliseconds)
                if expires:
                    time.sleep(CONCLUSION_SAMPLE_WAIT_PAST_SHORT_LEASE_SECONDS)
                before = stored_conclusion_keys(client)
            deleted = conclusions_command(server, "--repository-root", tree, "image-judgement")
            counted = conclusions_command(server, "--dry-run", "--repository-root", tree, "image-judgement")
            with environment_for(unix_environment(server)):
                after = stored_conclusion_keys(process_client())
            holder_line = f"PRUNE_SERVER_DELETE_CONCLUSIONS_HOLDER holder={holder.decode()} claims=1 "
            if claimed != [("claimed", None, None)]:
                problems.append(f"认领没成：{claimed}")
            if expires:
                if deleted[0] != 0 or holder_line in deleted[2]:
                    problems.append(f"租期过了的认领不该拦：删 退 {deleted[0]}、「{deleted[2].strip()[-200:]}」{deleted[3].strip()[-200:]}")
                if len(after) >= len(before):
                    problems.append("租期过了之后删，库里一把键也没少")
            else:
                if deleted[0] != EXIT_REFUSED or holder_line not in deleted[2] or "PRUNE_SERVER_DELETE_CONCLUSIONS " in deleted[2]:
                    problems.append(f"有持有者时删该退 {EXIT_REFUSED}、点名持有者、不打末行：退 {deleted[0]}、「{deleted[2].strip()[-300:]}」{deleted[3].strip()[-200:]}")
                if counted[0] != 0 or holder_line not in counted[2] or not counted[2].rstrip().endswith(" holders=1"):
                    problems.append(f"有持有者时 --dry-run 该退 0、照列持有者：退 {counted[0]}、「{counted[2].strip()[-300:]}」{counted[3].strip()[-200:]}")
                if after != before:
                    problems.append(f"有持有者时库不该动：键 {len(after)} 把（要 {len(before)} 把）")
                for arguments in (["no-such-kind"], [], ["image-judgement", "observer"], ["--dry-run"]):
                    misuse = conclusions_command(server, "--repository-root", tree, *arguments)
                    if misuse[0] != EXIT_USAGE:
                        problems.append(f"用法不对（{arguments}）该退 {EXIT_USAGE}，退 {misuse[0]}：{misuse[2].strip()[-200:]}")
        finally:
            server.stop()
    return not problems, ("；".join(problems) if problems else "租期没到的持有者拦删（点名它、库不动，--dry-run 照列）、租期过了的不拦，用法不对退 2")


def cell_delete_conclusions_refuses_a_source_that_differs(binary, work):
    """删之前对账（客户端文件头「命令行」delete-conclusions）：假源码树里 ConclusionBlocks 的表标签、块键长度各改一处、验证包的 verdict_store.rs 缺了：
    各退 1、点名对不上的那一处、库一行不少；一棵都对得上的 --dry-run 退 0（拒是因为对不上，不是别的）。"""
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        rows = conclusion_sample_rows()
        with environment_for(unix_environment(server)):
            client = process_client()
            write_conclusion_sample_rows(client, rows)
            before = stored_conclusion_keys(client)
        trees = (("ConclusionBlocks 的表标签", conclusion_source_tree(os.path.join(work, "tag"), blocks_tag="0F"), "ConclusionBlocks"),
                 ("块键长度", conclusion_source_tree(os.path.join(work, "length"), block_key_bytes=73), "块键长度"),
                 ("verdict_store.rs 缺了", conclusion_source_tree(os.path.join(work, "missing"), with_store_file=False), "读不了"))
        problems = []
        for what, tree, mentioned in trees:
            refused = conclusions_command(server, "--repository-root", tree, "image-judgement")
            with environment_for(unix_environment(server)):
                after = stored_conclusion_keys(process_client())
            if refused[0] != EXIT_REFUSED or mentioned not in refused[3] or "PRUNE_SERVER_DELETE_CONCLUSIONS " in refused[2] or after != before:
                problems.append(f"{what}对不上时该退 {EXIT_REFUSED}、点名「{mentioned}」、什么都不动：退 {refused[0]}、标准错误「{refused[3].strip()[-200:]}」、库{'动了' if after != before else '没动'}")
        agreeing = conclusions_command(server, "--dry-run", "--repository-root", conclusion_source_tree(os.path.join(work, "agreeing")), "image-judgement")
        if agreeing[0] != 0:
            problems.append(f"对得上的树 --dry-run 该退 0，退 {agreeing[0]}：{agreeing[3].strip()[-200:]}")
        return not problems, ("；".join(problems) if problems else "表标签、块键长度对不上与源码缺了都退 1、点名、库不动；对得上的 --dry-run 退 0")
    finally:
        server.stop()


def cell_conclusion_kinds_match_the_verdict_store(_binary, _work):
    """客户端抄的表标签、结论种类字节与块键长度对这一棵仓里的验证包源码（conclusion_source_problems(REPOSITORY_ROOT)）一个问题都没有。
    验证包的表在补丁里改了，要在补丁合进来之后的树里才绿；这一格不起服务。"""
    problems = conclusion_source_problems(REPOSITORY_ROOT)
    return not problems, ("；".join(problems) if problems else "客户端抄的表标签、结论种类字节与块键长度和验证包源码对得上")


def cell_temporary_service_binds_the_address_of_this_host(binary, work):
    """临时服务的 TCP 绑多机配置 HOSTS 里这一台的地址（让第二台的 GPU 批处理服务也连得到），不是回环：就绪行 tcp= 的地址正是 resolve 认出的这一台、不是回环地址、
    端口是内核挑的非 0 端口；没有 socket 文件、只设这个 TCP 地址，写一条读回一条，走的是 tcp。判别力：弄坏开关 temporary-service-binds-loopback。"""
    expected_address = this_host_address()
    server = TestServer(binary, os.path.join(work, "server"))
    try:
        bound_host, _separator, bound_port = server.tcp_address.rpartition(":")
        bound_host = bound_host.strip("[]")
        is_loopback = ipaddress.ip_address(bound_host).is_loopback
        with environment_for(tcp_environment(server)):
            client = process_client()
            client.write_many([(b"tcp/host-address", b"over-the-host-address")])
            value = client.read_many([b"tcp/host-address"])[0]
            route = client.route
        passed = (bound_host == expected_address and not is_loopback and bound_port.isdigit() and int(bound_port) > 0
                  and value == b"over-the-host-address" and route.startswith("tcp:"))
        return passed, (f"绑的是 {server.tcp_address}：地址是 HOSTS 里这一台的 {bound_host == expected_address}、回环 {is_loopback}（要不是）、"
                        f"端口 {bound_port}（要内核挑的非 0 端口）；经它走 TCP 读回 {value!r}，走的 {route}")
    finally:
        server.stop()


def cell_temporary_service_refuses_without_readable_hosts(binary, work):
    """多机配置读不到（SINGLEFS_MULTI_HOST_CONFIG 指到不在的文件）时 --with-temporary-service 退 78、一条命令都不起、说出读不到的是多机配置 HOSTS，不退回回环：
    子进程跑本文件的 --with-temporary-service，命令是写一个标记文件。判别力：弄坏开关 temporary-service-loopback-when-hosts-unreadable
    （读不到时退回 127.0.0.1:0 去起；这一格给的二进制不是服务，起不来退 2，不是 78）。"""
    marker = os.path.join(work, "command-ran")
    environment = dict(cell_environment({}), **{MULTI_HOST_CONFIGURATION_VARIABLE: os.path.join(work, "no-such-multi-host.env")})
    completed = subprocess.run([sys.executable, ENTRY_SCRIPT_PATH, "--with-temporary-service", "--server-binary", sys.executable, "--",
                                sys.executable, "-c", f"open({marker!r}, 'w').close()"],
                               capture_output=True, text=True, env=environment, timeout=CHILD_TIMEOUT_SECONDS)
    passed = completed.returncode == RUN_CONDITION_EXIT and "HOSTS" in completed.stderr and not os.path.exists(marker)
    return passed, (f"退 {completed.returncode}（要 {RUN_CONDITION_EXIT}）、标准错误点名 HOSTS {'HOSTS' in completed.stderr}、命令{'跑了' if os.path.exists(marker) else '没跑'}（要没跑）；"
                    f"标准错误末尾：{completed.stderr.strip()[-200:]!r}")
