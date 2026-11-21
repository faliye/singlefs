"""task_server_selftest.py 拆出的一份：第二台、NFS、共享根与路径判法的各格（总说明在 task_server_selftest.py 文件头）。"""
import os
import contextlib
import json
import re
import shlex
import subprocess
import threading
import time
import task_server_lib as lib
from task_server_selftest_fakes import GIBIBYTE, LOCAL_CARD_ZERO, PLACEMENT_WAIT_SECONDS, POLL_SECONDS, SERVER, STATE_TIMEOUT_SECONDS
from task_server_selftest_world import (
    SCOPE_DAILY, SelftestFailure, World, cargo_targets, describe, fake_cargo_extra, first_attempt, health_until, systemctl_start_count,
    write_executable,
)


VOLATILE_SAMPLE_PATH = "/tmp/task-server-selftest-volatile-output"   # outside-shared-path-refused：只出现在命令的参数里，命令不写它


def cell_peer_runs_in_shared_tree(world):
    """第二台在同一棵共享的树里跑（task_server.py 文件头 ③；records/2026-10-31-task-server巡检.md 问题 74）。本机放不下 4G、第二台放得下：
    派第二台的那一次，假 multi-host-run.sh 记下的参数里没有 --send，它在任务的工作目录（同一个绝对路径）里起了命令，交给命令的 CARGO_TARGET_DIR 在假第二台的本地目录下
    （不在假共享根下），env 那一段不带按台取值的 ROCKSDB_LIB_DIR，尝试的环境里没有树的句柄；改主工作区的工具（hook-swap.py）派得到第二台
    （共享根外的路径提交时就拒，由 outside-shared-path-refused 判）。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.start_server()
    problems = []
    seen = os.path.join(world.repo, "seen-on-peer")
    shared = os.path.realpath(world.shared_root)
    run_id = world.submit(["bash", "-c", 'pwd -P > "$1"; printf "%s\\n" "${CARGO_TARGET_DIR:-没设}" >> "$1"', "show", seen, "fake-peer-run"],
                          extra={"ROCKSDB_LIB_DIR": os.path.join(world.root, "rocksdb", "lib")}, cwd=world.repo)
    summary = world.finished(run_id, timeout=PLACEMENT_WAIT_SECONDS)
    attempt = (summary["attempts"] or [{}])[-1]
    try:
        with open(seen, encoding="utf-8") as handle:
            where, target = (handle.read().splitlines() + ["", ""])[:2]
    except OSError:
        where, target = "（命令没写出来）", ""
    arguments = next((line for line in world.calls() if line.startswith(f"peer-args task={run_id} ")), "")
    words = peer_env_words(world, run_id)
    try:
        with open(attempt.get("spec") or "", encoding="utf-8") as handle:
            handle_names = [name for name in json.load(handle).get("environment", {}) if "PEER_TREE" in name]
    except (OSError, ValueError):
        handle_names = ["（尝试的说明读不到）"]
    if attempt.get("host") != "peer" or summary["task"].get("outcome") != "succeeded":
        problems.append(f"本机放不下、第二台放得下：应当在第二台跑完，实际 {describe(summary)}、假第二台收到「{arguments[:300]}」")
    if "--send" in arguments.split() or "--fetch" in " ".join(arguments.split()) or handle_names:
        problems.append(f"派第二台不该拷树：参数里不该有 --send、--fetch 这一类，尝试的环境里不该有树的句柄，实际参数「{arguments[:300]}」、句柄变量 {handle_names}")
    if where != os.path.realpath(world.repo):
        problems.append(f"第二台那条命令应当在同一个绝对路径 {os.path.realpath(world.repo)} 里起，实际在「{where}」")
    if not target.startswith(world.peer_local + os.sep) or target == shared or target.startswith(shared + os.sep):
        problems.append(f"第二台那条命令拿到的 CARGO_TARGET_DIR 应当在第二台本地这一趟的目录下（不在共享根 {shared} 下），实际「{target}」")
    if "ROCKSDB_LIB_DIR=" in words:
        problems.append(f"按台取值的预编库变量不该带到第二台（交给 multi-host-run.sh 的 env 词里不该有 ROCKSDB_LIB_DIR=），实际「{words[:300]}」")
    writer_id = world.submit(["bash", "-c", "exit 0", "x", "scripts/gate/hook-swap.py", "fake-peer-run"], cwd=world.repo)
    try:
        writer = world.finished(writer_id, timeout=PLACEMENT_WAIT_SECONDS)
        if (writer["attempts"] or [{}])[-1].get("host") != "peer" or writer["task"].get("outcome") != "succeeded":
            problems.append(f"改主工作区的工具（hook-swap.py）应当派得到第二台跑完（改的是同一棵共享的树），实际 {describe(writer)}")
    except SelftestFailure as failure:
        problems.append(f"改主工作区的工具（hook-swap.py）应当派得到第二台跑完，实际等不到结局（{failure}）")
        world.client(["cancel", str(writer_id)])
    if problems:
        raise SelftestFailure("第二台在共享的树里跑：" + "；".join(problems), world.server_log_lines())
    return "派第二台的在同一个绝对路径里起、不带 --send 与树的句柄、编译目录在第二台本地、不带按台取值的变量；改主工作区的工具派得到第二台"


def cell_peer_run_failed_retried(world):
    world.resources(local=(30, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 40G 的只有第二台放得下
    world.seed_peak("selftest-peer-retry", 40 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=255,run"], extra={lib.KEY_VARIABLE: "selftest-peer-retry"}, cwd=world.repo)
    summary = world.finished(task_id)
    outcomes = [attempt.get("outcome") for attempt in summary["attempts"]]
    if outcomes != ["peer-run-failed", "succeeded"] or summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"第二台头一次没跑成（ssh 被重置这类，退 255）应当重排、第二次跑成，实际各次 {outcomes}、{describe(summary)}", world.server_log_lines())
    return "第二台头一次没跑成记 peer-run-failed、同一上限重排，第二次跑成"


def cell_source_repository_passed_to_command(world):
    """每次尝试经环境 TASK_SERVER_SOURCE_REPOSITORY 把提交方那一份仓的根交给命令：本机那一条读到服务所在仓的绝对路径，第二台那一条经 env 那一段带过去同一个值
    （任务 2653、2654：agent-cost.py 在第二台照脚本所在的树拼会话目录、退 2）。"""
    world.resources(local=(30, 30))   # 头一段没有第二台：本机那一条在本机起（派得到第二台的照余量挑台）
    world.start_server()
    expected = os.path.realpath(world.repo)
    seen = os.path.join(world.root, "seen-source")
    local_summary = world.finished(world.submit(["bash", "-c", f'printf "%s" "${{{lib.SOURCE_REPOSITORY_VARIABLE}:-unset}}" > "$1"', "show-source", seen]))
    try:
        with open(seen, encoding="utf-8") as handle:
            local_seen = handle.read()
    except OSError:
        local_seen = "（命令没写出来）"
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：这一条去第二台
    peer_id = world.submit(["bash", "-c", "exit 0", "peer-source"], cwd=world.repo)
    peer_summary = world.finished(peer_id, timeout=PLACEMENT_WAIT_SECONDS)
    calls = world.calls()
    marks = [index for index, line in enumerate(calls) if line.startswith(f"peer task={peer_id} ")]
    peer_words = calls[marks[0] + 1] if marks and marks[0] + 1 < len(calls) and calls[marks[0] + 1].startswith("peer-words ") else ""
    problems = []
    if local_seen != expected:
        problems.append(f"本机那一条：命令环境里的 {lib.SOURCE_REPOSITORY_VARIABLE} 应当是 {expected}，实际读到「{local_seen}」")
    if (peer_summary["attempts"] or [{}])[0].get("host") != "peer" or f"{lib.SOURCE_REPOSITORY_VARIABLE}={expected}" not in peer_words.split():
        problems.append(f"第二台那一条：交给 multi-host-run.sh 的 env 那一段应当带 {lib.SOURCE_REPOSITORY_VARIABLE}={expected}，实际 {describe(peer_summary)}、那一段「{peer_words[:400]}」")
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return f"本机与第二台那一条都拿到 {lib.SOURCE_REPOSITORY_VARIABLE}={expected}"


def peer_env_words(world, task_id):
    """假 multi-host-run.sh 给这条任务记的 peer-words 那一行（交给第二台的 env 那一段连同命令）；没有交空串。"""
    calls = world.calls()
    marks = [index for index, line in enumerate(calls) if line.startswith(f"peer task={task_id} ")]
    return calls[marks[0] + 1] if marks and marks[0] + 1 < len(calls) and calls[marks[0] + 1].startswith("peer-words ") else ""


def cell_attempt_label_passed_to_command(world):
    """每次尝试经环境 TASK_SERVER_ATTEMPT_LABEL 把这一次的尝试标签交给命令（两台都给）：两台的 scope 照它对到尝试上（task_server.py 的 server_pending、
    memory_peaks_lib.py 文件头「在跑的 scope」）。本机那一条读到的是它这一次尝试的 label，第二台那一条经 env 那一段带过去同一个值。"""
    world.resources(local=(30, 30))   # 头一段没有第二台：本机那一条在本机起
    world.start_server()
    seen = os.path.join(world.root, "seen-label")
    local_summary = world.finished(world.submit(["bash", "-c", f'printf "%s" "${{{lib.ATTEMPT_LABEL_VARIABLE}:-unset}}" > "$1"', "show-label", seen]))
    try:
        with open(seen, encoding="utf-8") as handle:
            local_seen = handle.read()
    except OSError:
        local_seen = "（命令没写出来）"
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：这一条去第二台
    peer_id = world.submit(["bash", "-c", "exit 0", "peer-label"], cwd=world.repo)
    peer_summary = world.finished(peer_id, timeout=PLACEMENT_WAIT_SECONDS)
    peer_words = peer_env_words(world, peer_id)
    local_label = (local_summary["attempts"] or [{}])[0].get("label")
    peer_attempt = (peer_summary["attempts"] or [{}])[0]
    problems = []
    if not local_label or local_seen != local_label:
        problems.append(f"本机那一条：命令环境里的 {lib.ATTEMPT_LABEL_VARIABLE} 应当是这一次尝试的 label「{local_label}」，实际读到「{local_seen}」")
    if peer_attempt.get("host") != "peer" or f"{lib.ATTEMPT_LABEL_VARIABLE}={peer_attempt.get('label')}" not in peer_words.split():
        problems.append(f"第二台那一条：交给 multi-host-run.sh 的 env 那一段应当带 {lib.ATTEMPT_LABEL_VARIABLE}=<这一次尝试的 label>，"
                        f"实际 {describe(peer_summary)}、那一段「{peer_words[:400]}」")
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return f"本机与第二台那一条都拿到 {lib.ATTEMPT_LABEL_VARIABLE}=<这一次尝试的 label>"


def cell_peer_admission_marked(world):
    """派第二台的那一次（task_server.py 文件头 ③「第二台」）：服务只给那一层 multi-host-run.sh --peer 设 lib.PEER_ADMITTED_VARIABLE=<任务号>.<第几次>，
    env 那一段不带到第二台那条命令；本机那一次的命令看不到它。"""
    world.resources(local=(30, 30))   # 头一段没有第二台：本机那一条在本机起
    world.start_server()
    seen = os.path.join(world.root, "seen-peer-admission")
    local_summary = world.finished(world.submit(["bash", "-c", f'printf "%s" "${{{lib.PEER_ADMITTED_VARIABLE}-没设}}" > "$1"', "show-admission", seen]))
    try:
        with open(seen, encoding="utf-8") as handle:
            local_seen = handle.read()
    except OSError:
        local_seen = "（命令没写出来）"
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：这一条去第二台
    peer_id = world.submit(["bash", "-c", "exit 0", "peer-admission"], cwd=world.repo)
    peer_summary = world.finished(peer_id, timeout=PLACEMENT_WAIT_SECONDS)
    calls = world.calls()
    peer_lines = world.calls_of(peer_id, "peer")
    marks = [index for index, line in enumerate(calls) if line.startswith(f"peer task={peer_id} ")]
    peer_words = calls[marks[0] + 1] if marks and marks[0] + 1 < len(calls) and calls[marks[0] + 1].startswith("peer-words ") else ""
    problems = []
    if first_attempt(local_summary).get("host") != "local" or local_seen != "没设":
        problems.append(f"本机那一条的命令不该看到 {lib.PEER_ADMITTED_VARIABLE}，实际读到「{local_seen}」、{describe(local_summary)}")
    if first_attempt(peer_summary).get("host") != "peer" or not peer_lines or not peer_lines[0].endswith(f" admitted={peer_id}.1"):
        problems.append(f"第二台那一条：multi-host-run.sh 那一层应当拿到 {lib.PEER_ADMITTED_VARIABLE}={peer_id}.1，实际 {describe(peer_summary)}、调用「{peer_lines}」")
    if lib.PEER_ADMITTED_VARIABLE in peer_words:
        problems.append(f"env 那一段不该带 {lib.PEER_ADMITTED_VARIABLE}（第二台那条命令里再带 --peer 要被拒），实际「{peer_words[:400]}」")
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return f"第二台那一层拿到 {lib.PEER_ADMITTED_VARIABLE}={peer_id}.1、env 那一段不带它，本机那一条的命令看不到它"


def cell_shared_roots_passed_to_command(world):
    """每次尝试经环境 SINGLEFS_DATA_ROOT、SINGLEFS_WORK_ROOT 交两个共享根（task_server_lib.py 文件头「共享根」；问题 73：第二台的 ~ 不是两台共享的那个目录）：
    本机那一条读到服务那一侧的真路径，第二台那一条经 env 那一段带过去同样的值（不在共享根下的提交时就拒，由 shared-roots-outside-refused 判）。"""
    world.resources(local=(30, 30))   # 头一段没有第二台：本机那一条在本机起（派得到第二台的照余量挑台）
    world.start_server()
    expected = {lib.DATA_ROOT_VARIABLE: os.path.realpath(world.data_root), lib.WORK_ROOT_VARIABLE: os.path.realpath(world.work_root)}
    seen = os.path.join(world.root, "seen-shared-roots")
    show = f'printf "%s\\n%s" "${{{lib.DATA_ROOT_VARIABLE}:-unset}}" "${{{lib.WORK_ROOT_VARIABLE}:-unset}}" > "$1"'
    local_summary = world.finished(world.submit(["bash", "-c", show, "show-shared-roots", seen]))
    try:
        with open(seen, encoding="utf-8") as handle:
            local_seen = dict(zip(lib.SHARED_ROOT_VARIABLES, handle.read().split("\n")))
    except OSError:
        local_seen = {}
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：这一条去第二台
    peer_id = world.submit(["bash", "-c", "exit 0", "peer-shared-roots"], cwd=world.repo)
    peer_summary = world.finished(peer_id, timeout=PLACEMENT_WAIT_SECONDS)
    peer_words = peer_env_words(world, peer_id).split()
    problems = []
    if (local_summary["attempts"] or [{}])[0].get("host") != "local" or local_seen != expected:
        problems.append(f"本机那一条：命令环境里的两个共享根应当是 {expected}，实际读到 {local_seen or '（命令没写出来）'}、{describe(local_summary)}")
    missing = [f"{name}={value}" for name, value in expected.items() if f"{name}={value}" not in peer_words]
    if (peer_summary["attempts"] or [{}])[0].get("host") != "peer" or missing:
        problems.append(f"第二台那一条：交给 multi-host-run.sh 的 env 那一段应当带 {'、'.join(missing) or '两个共享根'}，实际 {describe(peer_summary)}、那一段「{' '.join(peer_words)[:400]}」")
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return f"本机与第二台那一条都拿到 {expected}"


def cell_peer_disk_floor(world):
    """派第二台之前看它的盘余量（task_server_lib.py 文件头「挑机器」的「第二台盘余量」；问题 72：剩 353M 时拷树写满）：服务另起的 df 报的余量不到下限 → 第二台用不了，
    本机放不下就排着、排队原因写「第二台盘只剩」、写一条 peer-unavailable 告警（余量的数变了不另写）、/health 的 peer 一栏写明；报告里带的余量够了 → 照派第二台、/health 认回。"""
    floor = SERVER.DISK_FLOOR_BYTES
    low_values = (floor - GIBIBYTE, floor - 2 * GIBIBYTE)
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64, peer_df=f"     Avail\n{low_values[0]}\n")   # 本机放不下 4G：只派得到第二台
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "disk-floor"], cwd=world.repo)
    world.next_pass()
    world.next_pass()
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64, peer_df=f"     Avail\n{low_values[1]}\n")   # 余量又少了一点：原因照新的写，不另告警
    world.next_pass()
    world.next_pass()
    problems = []
    held = world.summary(task_id)
    why = held.get("waiting_for") or ""
    if held["task"].get("state") != "queued" or held["attempts"] or "第二台盘只剩" not in why:
        problems.append(f"第二台盘余量不到下限：应当不派第二台、本机放不下就排着、排队原因写「第二台盘只剩」，实际 {held['task'].get('state')}、"
                        f"尝试 {len(held['attempts'])} 次、排队原因「{why}」")
    alerted = [alert for alert in world.alerts() if alert.get("kind") == "peer-unavailable" and "第二台盘只剩" in alert.get("line", "")]
    if len(alerted) != 1:
        problems.append(f"盘余量不到下限应当恰好写一条 peer-unavailable 告警（余量的数变了不另写），实际 {len(alerted)} 条：{alerted}")
    _, health = world.api("GET", "/health")
    peer = health.get("peer") or {}
    shown = SERVER.gibibytes_text(low_values[1])
    if peer.get("available") is not False or f"第二台盘只剩 {shown}" not in str(peer.get("why")):
        problems.append(f"/health 的 peer 一栏应当写用不了、原因「第二台盘只剩 {shown}」，实际 {peer}")
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64, peer_disk_bytes=floor + GIBIBYTE)   # 报告里带的余量够了
    done = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
    if done["task"].get("outcome") != "succeeded" or (done["attempts"] or [{}])[-1].get("host") != "peer":
        problems.append(f"盘余量够了应当照派第二台跑完，实际 {describe(done)}")
    back = health_until(world, lambda health: (health.get("peer") or {}).get("available") is True, STATE_TIMEOUT_SECONDS)
    if (back.get("peer") or {}).get("available") is not True:
        problems.append(f"盘余量够了 /health 的 peer 一栏应当认回用得了，实际 {back.get('peer')}")
    if problems:
        raise SelftestFailure("第二台盘余量：" + "；".join(problems), world.server_log_lines())
    return (f"df 报的余量不到下限 {SERVER.gibibytes_text(floor)}：不派第二台、排队原因与 /health 写「第二台盘只剩」、告警一条；报告里带的余量够了照派第二台、/health 认回")


def cell_gate_stages_sent_to_peer(world):
    """点到 .claude/gate.d/ 的门禁阶段派得到第二台（没有只派本机的目录）；带判定库的那一种提交时就拒，由 verdict-library-family-refused 判。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：派得到第二台的去第二台
    world.start_server()
    plain = world.submit(["bash", "-c", "exit 0", ".claude/gate.d/doc-text.sh", "--check", "clock-times"], cwd=world.repo)
    plain_summary = world.finished(plain, timeout=PLACEMENT_WAIT_SECONDS)
    plain_host = (plain_summary["attempts"] or [{}])[0].get("host")
    if plain_host != "peer" or plain_summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"点到 .claude/gate.d/ 的门禁阶段应当派第二台跑完，实际 {describe(plain_summary)}", world.server_log_lines())
    return "点到 .claude/gate.d/ 的门禁阶段派到第二台跑完"


def cell_peer_command_exit_split(world):
    """问题 79：派到第二台的命令自己退 255（multi-host-run.sh 照真的那样打了第二台那条命令的结束行 exit=255）记 command-failed、退 255、不重排；
    命令自己退 3 而 multi-host-run.sh 收尾退了 255 的照命令的 3 记 command-failed；没有那一行的 255 照旧是 peer-run-failed。"""
    world.resources(local=(30, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-peer-exit", 40 * GIBIBYTE)   # 40G 的只有第二台放得下
    world.start_server()
    key = {lib.KEY_VARIABLE: "selftest-peer-exit"}
    cases = [("命令自己退 255", ["fake-peer-step-exit=255"], "command-failed", 255, 1),
             ("命令自己退 3、multi-host-run.sh 收尾退 255", ["fake-peer-step-exit=3", "fake-peer-exit=255"], "command-failed", 3, 1),
             ("multi-host-run.sh 自己退 255（没有结束行）", ["fake-peer-exit=255"], "peer-run-failed", 255, 1 + lib.PEER_RUN_RETRIES)]
    problems = []
    for name, words, outcome, code, attempts in cases:
        summary = world.finished(world.submit(["bash", "-c", "exit 0", *words], extra=key, cwd=world.repo))
        hosts = {attempt.get("host") for attempt in summary["attempts"]}
        if (summary["task"].get("outcome") != outcome or summary["task"].get("exit_code") != code or len(summary["attempts"]) != attempts or hosts != {"peer"}):
            problems.append(f"{name}：应当在第二台、记 {outcome}、退 {code}、尝试 {attempts} 次，实际 {describe(summary)}、在 {hosts}")
    if problems:
        raise SelftestFailure("第二台的退出码分层：" + "；".join(problems), world.server_log_lines())
    return "第二台命令自己退 255 记 command-failed 不重排、收尾的 255 不盖掉命令的码；没有结束行的 255 照旧 peer-run-failed 重排"


def cell_nfs_automount_recognized(world):
    """问题 84：「挂载」那一项触发自动挂载、看文件系统类型：第二台读回 nfs 过；读回 autofs（x-systemd.automount 的占位，NFS 没挂上，mountpoint -q 照样退 0）
    与 stat 读不到（自动挂载触发不上）都不过，出路写在第二台上 reset-failed、ls 触发再挂。"""
    os.environ["TASK_SERVER_SELFTEST_WORLD"] = world.world   # 假 ssh 往这一格的世界目录记调用、读演第二台的几个文件
    environment = world.server_environment
    type_file, exit_file = os.path.join(world.world, "fake-ssh-mount-type"), os.path.join(world.world, "fake-ssh-mount-exit")
    problems = []
    for name, kind, code, passes in (("NFS 挂着", "nfs", None, True), ("只剩自动挂载的占位", "autofs", None, False), ("自动挂载触发不上", None, "1", False)):
        for path, value in ((type_file, kind), (exit_file, code)):
            with contextlib.suppress(FileNotFoundError):
                os.remove(path)
            if value is not None:
                with open(path, "w", encoding="utf-8") as handle:
                    handle.write(value + "\n")
        verdict = lib.nfs_mount_verdict(world.shared_root, "selftest-peer", environment)
        if (verdict == lib.NFS_PASSED) != passes or (not passes and "reset-failed" not in verdict):
            problems.append(f"{name}：「挂载」那一项应当{'过' if passes else '不过、出路写 reset-failed 与触发再挂'}，实际「{verdict}」")
    if problems:
        raise SelftestFailure("NFS 挂载的判法：" + "；".join(problems))
    return "第二台读回 nfs 过；只剩自动挂载的占位、触发不上都不过，出路写在第二台 reset-failed、ls 触发再挂"


def cell_data_root_key_required(world):
    """多机配置没写 SHARED_DATA_ROOT（task_server_lib.py 文件头「共享数据根」）：服务不起、退 78、点名这个键；control start 起之前就退 78、出路点名往 multi-host.env 加
    SHARED_DATA_ROOT= 那一行、不起 systemctl start；control install --dry-run 同样退 78、点名这个键。"""
    config = os.path.join(world.root, "no-data-root-multi-host.env")
    World.write_config(config, world.state, world.runtime, world.key, world.builds, world.shared_root, data_root=False)
    problems = []
    code, output = world.serve_until_exit(config)
    if code != lib.RUN_CONDITION_EXIT or lib.DATA_ROOT_KEY not in output:
        problems.append(f"服务应当不起、退 78、stderr 点名 {lib.DATA_ROOT_KEY}，实际{'起来了' if code is None else f'退 {code}'}、「{output.strip()[-300:]}」")
    extra = {lib.MULTI_HOST_CONFIG_VARIABLE: config, "TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl"),
             "XDG_CONFIG_HOME": os.path.join(world.root, "xdg-config")}
    started = world.control(["start"], extra=extra)
    if (started.returncode != lib.RUN_CONDITION_EXIT or f"{lib.DATA_ROOT_KEY}=" not in started.stderr or lib.MULTI_HOST_CONFIG_NAME not in started.stderr
            or systemctl_start_count(world)):
        problems.append(f"control start 应当起之前就退 78、出路点名往 {lib.MULTI_HOST_CONFIG_NAME} 加 {lib.DATA_ROOT_KEY}= 那一行、不起 systemctl start，"
                        f"实际退 {started.returncode}、systemctl start 收到 {systemctl_start_count(world)} 回、stderr「{started.stderr.strip()[-300:]}」")
    dry = world.control(["install", "--dry-run"], extra=extra)
    if dry.returncode != lib.RUN_CONDITION_EXIT or lib.DATA_ROOT_KEY not in dry.stderr:
        problems.append(f"control install --dry-run 应当退 78、点名 {lib.DATA_ROOT_KEY}，实际退 {dry.returncode}、stderr「{dry.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("没写共享数据根：" + "；".join(problems), [output[-1500:]])
    return f"没写 {lib.DATA_ROOT_KEY}：服务退 78、control start 起之前退 78、install --dry-run 退 78，都点名这个键"


def cell_outside_shared_path_refused(world):
    """① 共享根外的路径提交时就拒（task_server.py 文件头 ②「提交时拒」①）：整个词、bash -c 词里、工作目录在共享根外的，与「名字=/tmp/…」的值，
    task submit 都退 3、不给任务号，说明点名那个路径与 NFS_SHARED_ROOTS、出路写改放某一段下（/tmp 的那一句另说重启会清），服务一条都没收、没起；
    路径都在共享根下（连同 /dev/null、/usr/ 这类两台相同的系统路径）的照常进队、本机放不下就派到第二台跑完。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：收下的那一条去第二台
    os.makedirs(world.outside_root, exist_ok=True)
    world.start_server()
    outside_word, outside_quoted = os.path.join(world.outside_root, "input.txt"), os.path.join(world.outside_root, "quoted.txt")
    outside_tail = os.path.join(world.outside_root, "tail.txt")
    cases = {"整个词是共享根外的路径": (["bash", "-c", "exit 0", "x", outside_word], world.repo, outside_word, ()),
             "bash -c 词里有共享根外的路径": (["bash", "-c", f"cat {outside_quoted} > /dev/null"], world.repo, outside_quoted, ()),
             "bash -c 词里变量展开的尾巴与写死的共享根外路径同在（点名写死的那个）": (["bash", "-c", f'cp "${{X:?}}/release/a" $(pwd)/b {outside_tail}'], world.repo, outside_tail, ()),
             "工作目录在共享根外": (["bash", "-c", "exit 0", "outside-cwd"], world.outside_root, os.path.realpath(world.outside_root), ()),
             "「名字=/tmp/…」的值": (["bash", "-c", "exit 0", f"OUT={VOLATILE_SAMPLE_PATH}"], world.repo, VOLATILE_SAMPLE_PATH, ("重启会清掉 /tmp",))}
    problems = []
    for name, (command, cwd, named, more) in cases.items():
        completed = world.client(["submit", "--cwd", cwd, "--", *command])
        wanted = (named, lib.NFS_ROOTS_KEY, "改放") + more
        if completed.returncode != 3 or lib.SUBMIT_MARK in completed.stdout or not all(text in completed.stderr for text in wanted):
            problems.append(f"{name}：应当退 3、不给任务号、说明点名 {named} 与 {lib.NFS_ROOTS_KEY}、出路写改放某一段下{'、另说重启会清 /tmp' if more else ''}，"
                            f"实际退 {completed.returncode}、stdout「{completed.stdout.strip()[-200:]}」、stderr「{completed.stderr.strip()[-400:]}」")
    listed = json.loads(world.client(["status", "--json"]).stdout or "{}").get("tasks", [])
    if listed or world.calls():
        problems.append(f"拒了的一条都不该进队、不该起，实际会话里列着 {[item.get('id') for item in listed]}、包装调用 {world.calls()[:5]}")
    # 变量展开、命令替换之后的尾巴不是写死的绝对路径：不拒（别的会话报的误拒）；写在 -c 脚本词里（照 shell 词读的只有它），exit 0 在前、这几段不执行
    expansion_tails = 'echo "${X:?}/release/a" $(pwd)/b ${X:-.}/c/d'
    inside = ["bash", "-c", f"exit 0; {expansion_tails}", "x", os.path.join(world.repo, "input.txt"), "/dev/null", "/usr/bin/true", "fake-peer-run"]
    try:
        summary = world.finished(world.submit(inside, cwd=world.repo), timeout=PLACEMENT_WAIT_SECONDS)
        if first_attempt(summary).get("host") != "peer" or summary["task"].get("outcome") != "succeeded":
            problems.append(f"路径都在共享根下的应当照收、本机放不下就派第二台跑完，实际 {describe(summary)}")
    except SelftestFailure as failure:
        problems.append(f"路径都在共享根下的应当照收、派第二台跑完，实际 {failure}")
    if problems:
        raise SelftestFailure("共享根外的路径提交时拒：" + "；".join(problems), world.server_log_lines())
    return ("整个词、bash -c 词里、工作目录在共享根外的与 /tmp 下的提交时退 3、点名那个路径，一条都没收；"
            "都在共享根下的、连同变量展开与命令替换之后的 /… 尾巴照收、派到第二台跑完")


SLASH_IN_ARGUMENT = "fs-meta 一项 （接的是 /）"   # path-check-whole-words：参数值中间夹着一个 /（交 crates-mutation-replay.sh 的 --item 那一种），不是路径


def cell_path_check_whole_words(world):
    """① 共享根外的路径只认整个 shell 词（task_server.py 文件头 ②「提交时拒」①；find_outside_shared_path）：参数值中间夹着 / 的（--item 后面那个词「…（接的是 /）」、
    --item=… 写在一个词里的、bash -c 脚本里引号括起来的同一段）照收、跑完；整个词是共享根外绝对路径的（--item 后面那个词、--output=<路径> 的值）照拒：
    退 3、不给任务号、点名那个路径与 NFS_SHARED_ROOTS。弄坏开关 path-check-substring 下带空白的词都按子串认，照收的那三条被拒，这一格红。"""
    world.resources(local=(30, 30))
    os.makedirs(world.outside_root, exist_ok=True)
    world.start_server()
    outside = os.path.join(world.outside_root, "whole-word.txt")
    accepted = {"--item 后面那个词中间夹着 /": ["bash", "-c", "exit 0", "slash-argument", "--item", SLASH_IN_ARGUMENT],
                "--item=… 一个词里夹着 /": ["bash", "-c", "exit 0", "slash-assignment", f"--item={SLASH_IN_ARGUMENT}"],
                "bash -c 脚本里引号括起来的一段夹着 /": ["bash", "-c", f'exit 0; echo "{SLASH_IN_ARGUMENT}"', "slash-quoted"]}
    refused = {"--item 后面整个词是共享根外的路径": ["bash", "-c", "exit 0", "whole-word", "--item", outside],
               "--output=<共享根外的路径> 的值": ["bash", "-c", "exit 0", "option-value", f"--output={outside}"]}
    problems = []
    for name, command in accepted.items():
        completed = world.client(["submit", "--cwd", world.repo, "--", *command])
        found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
        if completed.returncode != 0 or not found:
            problems.append(f"{name}：参数里夹着的 / 不是路径，应当照收（退 0、打任务号），实际退 {completed.returncode}、stderr「{completed.stderr.strip()[-300:]}」")
            continue
        world.confirm_submitted(completed)
        summary = world.finished(int(found.group(1)))
        if summary["task"].get("outcome") != "succeeded":
            problems.append(f"{name}：照收之后应当跑完，实际 {describe(summary)}")
    for name, command in refused.items():
        completed = world.client(["submit", "--cwd", world.repo, "--", *command])
        if completed.returncode != 3 or lib.SUBMIT_MARK in completed.stdout or outside not in completed.stderr or lib.NFS_ROOTS_KEY not in completed.stderr:
            problems.append(f"{name}：应当退 3、不给任务号、说明点名 {outside} 与 {lib.NFS_ROOTS_KEY}，"
                            f"实际退 {completed.returncode}、stdout「{completed.stdout.strip()[-200:]}」、stderr「{completed.stderr.strip()[-400:]}」")
    if problems:
        raise SelftestFailure("共享根外的路径只认整个 shell 词：" + "；".join(problems), world.server_log_lines())
    return f"参数值中间夹着 / 的 {len(accepted)} 条照收、跑完；整个词是共享根外路径的 {len(refused)} 条退 3、点名那个路径"


def cell_shared_roots_outside_refused(world):
    """② 交给命令的共享根不在共享根下提交时就拒（task_server.py 文件头 ②「提交时拒」②）：任务自己带的 SINGLEFS_DATA_ROOT、服务的工作根 SINGLEFS_WORK_ROOT
    在 NFS_SHARED_ROOTS 外的，task submit 退 3、不给任务号，说明点名变量与值、出路写配多机配置，一条都没收；两个都在共享根下的照常进队、本机放不下就派到第二台跑完。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：收下的那一条去第二台
    os.makedirs(world.outside_root, exist_ok=True)
    outside = os.path.realpath(world.outside_root)
    world.start_server()
    problems = []

    def refused(name, extra, named):
        completed = world.client(["submit", "--cwd", world.repo, "--", "bash", "-c", "exit 0", "shared-roots-outside"], extra=extra)
        wanted = (named, lib.NFS_ROOTS_KEY, "配多机配置")
        if completed.returncode != 3 or lib.SUBMIT_MARK in completed.stdout or not all(text in completed.stderr for text in wanted):
            problems.append(f"{name}：应当退 3、不给任务号、说明点名 {named} 与 {lib.NFS_ROOTS_KEY}、出路写配多机配置，"
                            f"实际退 {completed.returncode}、stderr「{completed.stderr.strip()[-400:]}」")

    refused("任务带的数据根在共享根外", {lib.DATA_ROOT_VARIABLE: outside}, f"{lib.DATA_ROOT_VARIABLE}={outside}")
    try:
        summary = world.finished(world.submit(["bash", "-c", "exit 0", "shared-roots-inside"], cwd=world.repo), timeout=PLACEMENT_WAIT_SECONDS)
        if first_attempt(summary).get("host") != "peer" or summary["task"].get("outcome") != "succeeded":
            problems.append(f"两个共享根都在共享根下的应当照收、派第二台跑完，实际 {describe(summary)}")
    except SelftestFailure as failure:
        problems.append(f"两个共享根都在共享根下的应当照收、派第二台跑完，实际 {failure}")
    world.stop_server()
    world.start_server(extra={lib.WORK_ROOT_VARIABLE: outside})
    refused("服务的工作根在共享根外", None, f"{lib.WORK_ROOT_VARIABLE}={outside}")
    peer_runs = [line for line in world.calls() if line.startswith("peer task=")]
    if len(peer_runs) != 1:
        problems.append(f"拒了的两条都不该派出去（第二台只该收到收下的那一条），实际第二台收到 {peer_runs}")
    if problems:
        raise SelftestFailure("共享根不在共享根下提交时拒：" + "；".join(problems), world.server_log_lines())
    return f"任务带的 {lib.DATA_ROOT_VARIABLE}、服务的 {lib.WORK_ROOT_VARIABLE} 在共享根外时提交退 3、点名变量与值；都在共享根下的照收、派到第二台跑完"


NFS_PROBE_READ_DELAY_SECONDS = 8   # nfs-probe-concurrent-runs：头一趟 control check 的第二台读探针那一步先睡这么久，另一趟在这段时间里整个跑完（一趟 control check 一两秒，推的）


def cell_nfs_probe_concurrent_runs(world):
    """两趟 control check --probe 交错跑两向探针（task_server_lib.py 的 nfs_probe）：头一趟写好本机探针、第二台读它之前先睡（假 ssh 的 fake-ssh-read-delay），
    另一趟在这段时间里整个跑完、收尾删自己的探针；头一趟醒来照样读得到它自己写的那一份，两趟都报 NFS 共享过。"""
    world.start_server()
    delay = os.path.join(world.world, "fake-ssh-read-delay")
    with open(delay, "w", encoding="utf-8") as handle:
        handle.write(f"{NFS_PROBE_READ_DELAY_SECONDS}\n")
    earlier = world.control(["check", "--probe"], background=True)
    patience = world.patience(STATE_TIMEOUT_SECONDS)
    while not os.path.exists(delay + ".taken") and earlier.poll() is None and patience.waiting():
        time.sleep(POLL_SECONDS)
    later = world.control(["check", "--probe"])
    try:
        earlier_output, earlier_error = world.communicate(earlier, STATE_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        earlier_output, earlier_error = world.stop_client(earlier)
    problems = []
    if not os.path.exists(delay + ".taken"):
        problems.append("头一趟 control check 的第二台没读到探针就收尾了（假 ssh 没走到读探针那一步）")
    for name, code, output in (("头一趟", earlier.returncode, earlier_output + earlier_error), ("另一趟", later.returncode, later.stdout + later.stderr)):
        if code != 0 or "NFS 共享 过" not in output:
            problems.append(f"{name} control check 应当退 0、说 NFS 共享过，实际退 {code}、「{output.strip()[-300:]}」")
    left = sorted(name for name in os.listdir(world.shared_root) if name.startswith((lib.NFS_PROBE_NAME, lib.NFS_PEER_PROBE_NAME)))
    if left:
        problems.append(f"两趟收尾之后探针文件应当都删了，实际还在 {left}")
    if problems:
        raise SelftestFailure("两趟两向探针交错：" + "；".join(problems), world.server_log_lines())
    return "两趟 control check 的两向探针交错：先写的那一趟等另一趟整个跑完收尾之后照样读到自己的探针，两趟都过、探针都删了"


FAKE_RECOVER_CLEANUP = r'''#!/usr/bin/env bash
# peer-cleanup-recovered-once 的假 multi-host-run.sh：只认 --recover-cleanup <回执>：往回执同目录的 recover-calls 记一行，睡 1 秒（演连第二台停、删那一趟），
# 往标准输出打一行 selftest-recovered，再把回执原子写成 confirmed
[[ "${1:-}" == --recover-cleanup ]] || { echo "假 multi-host-run.sh：只认 --recover-cleanup" >&2; exit 2; }
receipt="$2"
printf 'recover %s\n' "$receipt" >> "$(dirname -- "$receipt")/recover-calls"
sleep 1
echo "selftest-recovered $receipt"
printf '{"status": "confirmed"}\n' > "$receipt.partial" && mv -- "$receipt.partial" "$receipt"
'''


def cell_peer_cleanup_recovered_once(world):
    """派第二台的一次尝试清理回执是 pending，两个线程（演对账与停单元的后台线程）同一刻确认它（task_server.py 的 confirm_peer_cleanup）：
    --recover-cleanup 只起一趟、那一次的日志只追加一份，后到的等前一趟放下、读到 confirmed，两边都交确认了；锁用完从锁表（PEER_CLEANUP_LOCKS）删。
    恢复脚本是假的（FAKE_RECOVER_CLEANUP：睡 1 秒再写 confirmed，两边同时起的话都读得到 pending）；只用 task_server.py 的 confirm_peer_cleanup，
    套在只备环境与恢复脚本的假调度器上，不起服务。弄坏开关 peer-cleanup-recovered-concurrently 下两趟都起，这一格红。"""
    directory = os.path.join(world.root, "peer-cleanup")
    os.makedirs(directory)
    receipt, log = os.path.join(directory, "peer-cleanup.json"), os.path.join(directory, "attempt.log")
    with open(receipt, "w", encoding="utf-8") as handle:
        json.dump({"status": "pending"}, handle)
    runner = os.path.join(world.bin, "fake-recover-cleanup")
    write_executable(runner, FAKE_RECOVER_CLEANUP)
    scheduler = object.__new__(SERVER.Scheduler)
    scheduler.environment = dict(os.environ)
    scheduler.peer_runner = runner
    attempt = {"id": 7, "task_id": 7, "number": 1, "host": SERVER.PEER, "peer_cleanup_file": receipt, "log": log}
    barrier = threading.Barrier(2)
    results = {}

    def confirm(name):
        barrier.wait(timeout=10)
        results[name] = SERVER.Scheduler.confirm_peer_cleanup(scheduler, dict(attempt))
    threads = [threading.Thread(target=confirm, args=(name,), daemon=True) for name in ("对账", "停单元的后台线程")]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join(timeout=STATE_TIMEOUT_SECONDS)
    calls_path = os.path.join(directory, "recover-calls")
    calls = open(calls_path, encoding="utf-8").read().splitlines() if os.path.exists(calls_path) else []
    logged = [line for line in (open(log, encoding="utf-8").read().splitlines() if os.path.exists(log) else []) if line.startswith("selftest-recovered ")]
    alive = [thread.name for thread in threads if thread.is_alive()]
    if alive or len(calls) != 1 or len(logged) != 1 or sorted(results.values()) != [(True, ""), (True, "")] or SERVER.PEER_CLEANUP_LOCKS:
        raise SelftestFailure(f"两个线程同时确认同一次尝试的清理：--recover-cleanup 应当只起一趟、日志只追加一份、两边都交确认了、锁表空着，"
                              f"实际起了 {len(calls)} 趟、日志 {len(logged)} 份、两边交 {results}、还没退的线程 {alive}、锁表 {list(SERVER.PEER_CLEANUP_LOCKS)}")
    return "两个线程同时确认同一次尝试的清理：恢复只起一趟、日志一份，后到的读到 confirmed；锁表空着"


CARD_GATE_ENTRY = ".claude/gate.d/crash-point-replay.sh"   # attempt-gets-no-card：假仓根下的假门禁（把自己的环境写进一份文件，不起真门禁）
CARD_CHOICE_SAMPLE = {"GPU_CARDS_BY_PRIORITY": LOCAL_CARD_ZERO, "CUDA_VISIBLE_DEVICES": "0"}   # 那一格服务自己的环境里带着的选卡变量
CARD_CHANNEL_PREFIX = "TASK_SERVER_CARD_LEASE"   # 领卡通道交给命令的变量与客户端打的领卡行都以它起头：命令的环境与尝试的日志里都不该有


def cell_attempt_gets_no_card(world):
    """任务不拿卡（task_server.py 的 start_attempt、attempt_command；.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：服务自己的环境里带着
    GPU_CARDS_BY_PRIORITY 与 CUDA_VISIBLE_DEVICES，交一条点名的假门禁 crash-point-replay（假仓根下的脚本，把自己的环境写进一份文件，不起真门禁）：
    命令的环境里两个选卡变量与 CARD_CHANNEL_PREFIX 起头的都没有；尝试的说明里起的那一条外面没套清场包装（gpu-quiesce-run.sh）；
    尝试的日志里没有 CARD_CHANNEL_PREFIX 起头的行；任务跑完、退 0。弄坏开关 card-variables-passed-to-attempt 下服务环境里的选卡变量照交给命令，这一格红。"""
    seen = os.path.join(world.root, "card-gate-environment")
    entry = os.path.join(world.repo, CARD_GATE_ENTRY)
    os.makedirs(os.path.dirname(entry), exist_ok=True)
    write_executable(entry, f'#!/usr/bin/env bash\nenv > {shlex.quote(seen)}\necho "假门禁：$*"\nexit 0\n')
    world.resources(local=(30, 30))
    world.start_server(extra=CARD_CHOICE_SAMPLE)
    task_id = world.submit(["bash", CARD_GATE_ENTRY, ".", "--check", "crash-case-replay", "--crash-cases-only", "--item", "crash-case:selftest"],
                           extra={"SINGLEFS_RUN_SCOPE": SCOPE_DAILY}, cwd=world.repo)
    summary = world.finished(task_id)
    attempt = first_attempt(summary)
    problems = []
    if summary["task"].get("outcome") != "succeeded":
        problems.append(f"假门禁应当照跑完、退 0，实际 {describe(summary)}")
    try:
        with open(seen, encoding="utf-8") as handle:
            names = {line.split("=", 1)[0] for line in handle if "=" in line}
        leaked = sorted(name for name in names if name in lib.CARD_CHOICE_VARIABLES or name.startswith(CARD_CHANNEL_PREFIX))
        if leaked:
            problems.append(f"命令的环境里不该有选卡变量与领卡通道，实际有 {leaked}")
    except OSError as error:
        problems.append(f"假门禁没写下它的环境（{error}）")
    try:
        with open(attempt.get("spec") or "", encoding="utf-8") as handle:
            argv = json.load(handle).get("argv") or []
        wrapped = [word for word in argv if os.path.basename(str(word)) == "gpu-quiesce-run.sh"]
        if not argv or wrapped:
            problems.append(f"起的那一条外面不该套清场包装，实际说明里的 argv {argv[:8]}")
    except (OSError, ValueError) as error:
        problems.append(f"尝试的说明读不了（{error}）")
    with contextlib.suppress(OSError):
        with open(attempt.get("log") or "", encoding="utf-8", errors="replace") as handle:
            marked = [line.strip() for line in handle if line.startswith(CARD_CHANNEL_PREFIX)]
        if marked:
            problems.append(f"尝试的日志里不该有 {CARD_CHANNEL_PREFIX} 起头的行，实际 {marked[:3]}")
    if problems:
        raise SelftestFailure("任务不拿卡：" + "；".join(problems), world.server_log_lines())
    return "假门禁照跑完：命令的环境里没有选卡变量与领卡通道、外面没套清场、日志里没有领卡行"


def cell_peer_retry_build_directory_reused(world):
    """派到第二台的会编命令撞上限重排、第 2 次还在第二台：编译目录是第 1 次那一个（task_server.py 的 take_build_slot、PeerMachine.command；
    假 multi-host-run.sh 照真的那样认 --keep-build-directory 与 --reuse-build-directory）：第 1 次带 --keep-build-directory、留着；第 2 次带 --reuse-build-directory <第 1 次的标签>、
    记 build_directory_reused，假 cargo 拿到的 CARGO_TARGET_DIR 是那一个下面的 target；有了结局之后它照旧留在这棵树在第二台的组里（不经 --remove-build-directory 删）。
    弄坏开关 peer-retry-build-directory-fresh 下第二台每一次新建、跑完删，build-slot-per-attempt 下第 2 次新开一个，这一格红。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：两次都派第二台
    world.start_server()
    extra = fake_cargo_extra(world)
    summary = world.finished(world.submit(["cargo", "build", "fake-peer-run", "fake-wrapper-exits=250,run"], extra=extra, cwd=world.repo),
                             timeout=PLACEMENT_WAIT_SECONDS)
    attempts = summary["attempts"]
    hosts = [attempt.get("host") for attempt in attempts]
    directories = [attempt.get("build_directory") or "" for attempt in attempts]
    labels = [attempt.get("label") or "" for attempt in attempts]
    task_id = summary["task"]["id"]
    arguments = [line for line in world.calls() if line.startswith(f"peer-args task={task_id} ")]
    targets = cargo_targets(world)
    problems = []
    if (summary["task"].get("outcome") != "succeeded" or hosts != ["peer", "peer"] or not all(directories) or directories[0] != directories[1]
            or not attempts[1].get("build_directory_reused") or os.path.basename(directories[0]) != labels[0]):
        problems.append(f"第二台先退 250 再跑：第 2 次的编译目录应当是第 1 次那一个（builds/<第 1 次的标签>）、记 build_directory_reused，"
                        f"实际 {describe(summary)}、各次在 {hosts}、编译目录 {directories}、标签 {labels}、"
                        f"第 2 次 build_directory_reused={attempts[-1].get('build_directory_reused') if attempts else None}")
    if (len(arguments) != 2 or "--keep-build-directory" not in arguments[0].split()
            or f"--reuse-build-directory {labels[0] if labels else ''}" not in arguments[1]):
        problems.append(f"交给 multi-host-run.sh 的参数：第 1 次应当带 --keep-build-directory，第 2 次应当带 --reuse-build-directory <第 1 次的标签>，实际 {[line[:300] for line in arguments]}")
    if directories and targets != [os.path.join(directories[0], "target")]:
        problems.append(f"假 cargo 拿到的 CARGO_TARGET_DIR 应当是第 1 次那一个下面的 target（{os.path.join(directories[0], 'target') if directories else ''}），实际 {targets}")
    removals = [line for line in world.calls() if line.startswith("peer-remove ")]
    if not directories or not directories[0] or not os.path.isdir(os.path.join(directories[0], "target")) or removals:
        problems.append(f"有了结局之后第二台那一个应当照旧留在组里（不经 --remove-build-directory 删），实际那一个"
                        f"{'在' if directories and directories[0] and os.path.isdir(os.path.join(directories[0], 'target')) else '不在了'}、删的调用 {removals}")
    if problems:
        raise SelftestFailure("第二台重试的编译目录：" + "；".join(problems), world.server_log_lines())
    return "第二台重排的那一次接着用第 1 次留着的编译目录；有了结局之后它照旧留在组里"
