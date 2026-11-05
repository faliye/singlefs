#!/usr/bin/env python3
"""E171 各层编排器自证用的假 task-server 分片（task-server/task_server_lib.py 文件头「分片」那三步，照真的环境变量）：
  fake_task_server.py --root <目录> [--roles local,peer] [--peer-without rsync] -- <命令…>
在 <目录>/run-<序>/ 下照真的那样跑三步：① 计划：TASK_SERVER_SHARD_PHASE=plan、TASK_SERVER_SHARD_PLAN_DIRECTORY=<序目录>/plan，读它写的 plan.json
（片里只认 label 一个键，别的键退 234，同服务的 shard-plan-refused）；② 各片并行：TASK_SERVER_SHARD_PHASE=shard、TASK_SERVER_SHARD=<片号>/<片数>、
TASK_SERVER_SHARD_OUTPUT_DIRECTORY=<序目录>/output-<片号>、TASK_SERVER_SHARD_HOST=<--roles 里第 片号 个（轮着取）那一台的假地址：local 是 192.0.2.1、peer 是 192.0.2.2，同真服务给 HOSTS 里那一台的地址>；
落在 peer 的那一片另给 MULTI_HOST_RUN_PEER_SCRATCH=<序目录>/peer-scratch-<片号>（服务派到第二台时给的那一个）；
每片另给 FAKE_DRIVER_HOST（local 记 1、peer 记 2，假逐行驱动照它记台号）；--peer-without rsync 时落在 peer 的那几片的 PATH 只剩
python3、env、nice、bash（第二台没装 rsync，预检照真的拒）；③ 各片都退 0 之后合并：TASK_SERVER_SHARD_PHASE=merge、TASK_SERVER_SHARD_OUTPUTS=<一份 TSV>。
各步的输出依次转到标准输出；退出码同服务：哪一步没成就是那一步的码（片里取片号最小的那一片），都成了是合并的码。
收到 TERM：对各片的进程组各发一次 TERM，等它们退出，退 143。再收到一次 TERM、或等了 GRACE_SECONDS 秒还有片没退：对各片的进程组发 KILL
（各片自己另起的进程不在这些组里，由自证的收尾收）。
"""
import json
import os
import shutil
import signal
import subprocess
import sys
import time

GRACE_SECONDS = 60.0
PEER_TOOLS = ("python3", "env", "nice", "bash")
HOST_NUMBERS = {"local": "1", "peer": "2"}
HOST_ADDRESSES = {"local": "192.0.2.1", "peer": "192.0.2.2"}   # 文档用的地址段（RFC 5737），不是哪一台的真地址


def next_run_directory(root):
    os.makedirs(root, exist_ok=True)
    numbers = [int(name.split("-", 1)[1]) for name in os.listdir(root) if name.startswith("run-") and name.split("-", 1)[1].isdigit()]
    path = os.path.join(root, f"run-{max(numbers, default=0) + 1}")
    os.makedirs(os.path.join(path, "plan"))
    return path


def peer_path(run_directory):
    directory = os.path.join(run_directory, "peer-bin")
    os.makedirs(directory, exist_ok=True)
    for name in PEER_TOOLS:
        found = sys.executable if name == "python3" else shutil.which(name)
        if found:
            os.symlink(found, os.path.join(directory, name))
    return directory


def main(arguments):
    split = arguments.index("--")
    own, command = arguments[:split], arguments[split + 1:]
    options = dict(zip(own[::2], own[1::2]))
    roles = options.get("--roles", "local,peer").split(",")
    run_directory = next_run_directory(options["--root"])
    plan = os.path.join(run_directory, "plan")
    terminated, started = [], []

    def signal_shards(signal_number):
        for process in list(started):
            try:
                os.killpg(process.pid, signal_number)
            except ProcessLookupError:
                pass

    def on_term(signal_number, frame):
        terminated.append(time.monotonic())
        signal_shards(signal.SIGTERM if len(terminated) == 1 else signal.SIGKILL)

    signal.signal(signal.SIGTERM, on_term)
    base = dict(os.environ, TASK_SERVER_TASK="fake-task-server", TASK_SERVER_SHARD_PLAN_DIRECTORY=plan)
    completed = subprocess.run(command, env=dict(base, TASK_SERVER_SHARD_PHASE="plan"), stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    sys.stdout.write(completed.stdout)
    if completed.returncode != 0:
        return completed.returncode
    try:
        shards = json.load(open(os.path.join(plan, "plan.json"), encoding="utf-8"))["shards"]
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"假 task-server：plan.json 读不通（{type(error).__name__}）")
        return 234
    if any(not isinstance(shard, dict) or set(shard) - {"label"} for shard in shards):
        print(f"假 task-server：计划里的片不是只带 label 的对象：{shards}")
        return 234
    count = len(shards)
    logs, outputs = [], []
    for index in range(count):
        role = roles[index % len(roles)]
        output = os.path.join(run_directory, f"output-{index}")
        environment = dict(base, TASK_SERVER_SHARD_PHASE="shard", TASK_SERVER_SHARD=f"{index}/{count}", TASK_SERVER_SHARD_OUTPUT_DIRECTORY=output,
                           TASK_SERVER_SHARD_HOST=HOST_ADDRESSES[role], FAKE_DRIVER_HOST=HOST_NUMBERS[role])
        if role == "peer":
            scratch = os.path.join(run_directory, f"peer-scratch-{index}")
            os.makedirs(scratch)
            environment["MULTI_HOST_RUN_PEER_SCRATCH"] = scratch
            if options.get("--peer-without") == "rsync":
                environment["PATH"] = peer_path(run_directory)
        log = open(os.path.join(run_directory, f"shard-{index}.log"), "w", encoding="utf-8")
        logs.append(log)
        outputs.append((index, role, output))
        if not terminated:
            started.append(subprocess.Popen(command, env=environment, stdout=log, stderr=subprocess.STDOUT, start_new_session=True))
    killed = False
    while any(process.poll() is None for process in started):
        if terminated and not killed and time.monotonic() - terminated[0] > GRACE_SECONDS:
            signal_shards(signal.SIGKILL)
            killed = True
        time.sleep(0.05)
    codes = [process.wait() for process in started]
    for log in logs:
        log.close()
    for index in range(len(started)):
        sys.stdout.write(open(os.path.join(run_directory, f"shard-{index}.log"), encoding="utf-8").read())
    sys.stdout.flush()
    if terminated:
        return 143
    for code in codes:
        if code != 0:
            return code if code >= 0 else 128 - code
    table = os.path.join(run_directory, "outputs.tsv")
    with open(table, "w", encoding="utf-8") as handle:
        handle.writelines(f"{index}\t{role}\t{output}\n" for index, role, output in outputs)
    completed = subprocess.run(command, env=dict(base, TASK_SERVER_SHARD_PHASE="merge", TASK_SERVER_SHARD_OUTPUTS=table),
                               stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    sys.stdout.write(completed.stdout)
    return completed.returncode


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
