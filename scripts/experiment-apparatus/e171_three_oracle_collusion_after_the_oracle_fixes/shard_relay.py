#!/usr/bin/env python3
"""E171 多机编排经 task-server 分片时的转交（rows_orchestrator.py 文件头「经 task-server」）：编排那一片要在别的台起的命令，不自己连那一台，
写进共享的计划目录交给服务派到那一台的那一片代跑。计划目录在共享工作根下（task-server/task_server_lib.py 文件头「分片」），两台照同一个绝对路径读写。

  shard_relay.py relay --plan <计划目录> --to <片号> --label <标签> [--poll <秒>] -- <命令…>
      编排那一片起它，当作那一台上的一条命令：写好请求，等代跑那一片写出退出码，把命令的标准输出与标准错误原样转出来，照它的退出码退出。
      收到 TERM / INT / HUP 写停止请求，代跑那一片对那条命令的进程组发 TERM，等它写出退出码再退。
      退出码：命令的退出码；2 用法错或标签撞了（命令没起）；255 代跑那一片已退出（写了 gone）或它的心跳停了 STALE_SECONDS 秒（命令结局不明，那一行当没跑成）。
  代跑的一面（serve）由编排器在片号不是 0 的那几片里调，不单独起。

计划目录下 relay/ 里的文件（都是排他新建或先写 .partial 再改名，两台各只写自己那几样）：
  shard-<片号>.json            各片登记：{"shard": 片号, "host": 那一片所在那一台的地址}（TASK_SERVER_SHARD_HOST，多机配置 HOSTS 里那一项）
  shard-<片号>.beat            各片心跳：一个整数，每 BEAT_SECONDS 秒加一；读的一方照自己的单调钟看它多久没变，不比两台的钟
  shard-<片号>.gone            代跑那一片退出时写（正常收尾、收到停止信号、心跳判停都写；写在它写完手上各条的退出码之后）；重试同一片登记时删。
                               转交的一方见到它、自己那条请求还没有退出码，就不会再有人代跑：立刻退 255，不等心跳停满
  finished                     编排那一片收尾时写；代跑的片见到它、手上的命令都跑完就退 0
  to-<片号>/<标签>/request.json  转交的命令：{"argv": […], "cwd": 目录}
  to-<片号>/<标签>/taken         代跑那一片认领（mkdir，只起一次）
  to-<片号>/<标签>/stop          转交的一方收到停止信号时写
  to-<片号>/<标签>/stdout、stderr、exit   命令的输出与退出码
代跑那一片照服务派到第二台那一趟的样子给每条命令一个这一台本地的目录：MULTI_HOST_RUN_PEER_SCRATCH=<本地目录>/<标签>、
  CARGO_TARGET_DIR=<那个目录>/target，命令跑完删掉。本地目录取这一片环境里的 MULTI_HOST_RUN_PEER_SCRATCH（服务派到第二台时给的），没有就在系统临时目录下新建一个。
编排那一片的心跳停了 STALE_SECONDS 秒（进程被硬杀，没写 finished）：代跑那一片停掉手上的命令、退 1。
"""
from __future__ import annotations

import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time

BEAT_SECONDS = 5.0
STALE_SECONDS = 900.0
SCRATCH_VARIABLE = "MULTI_HOST_RUN_PEER_SCRATCH"
EXIT_USAGE = 2
EXIT_TRANSPORT = 255
EXIT_STOPPED_BY_SIGNAL = 143
EXIT_ORCHESTRATOR_GONE = 1


def relay_root(plan: str) -> str:
    return os.path.join(plan, "relay")


def inbox(plan: str, shard: int) -> str:
    return os.path.join(relay_root(plan), f"to-{shard}")


def finished_path(plan: str) -> str:
    return os.path.join(relay_root(plan), "finished")


def gone_path(plan: str, shard: int) -> str:
    return os.path.join(relay_root(plan), f"shard-{shard}.gone")


def write_replacing(path: str, text: str) -> None:
    """先写 <路径>.partial 再改名：读的一方要么读不到、要么读到整份。"""
    partial = path + ".partial"
    with open(partial, "w", encoding="utf-8") as handle:
        handle.write(text)
    os.replace(partial, path)


def is_host_address(text) -> bool:
    """TASK_SERVER_SHARD_HOST 的值像不像多机配置 HOSTS 里的一项：非空、不带空白；地址照记，不另认角色名。"""
    return isinstance(text, str) and bool(text) and not any(character.isspace() for character in text)


def register(plan: str, shard: int, host_address: str) -> None:
    """登记这一片；服务重试同一片（同一个计划目录）时照新的那一次重写。"""
    os.makedirs(relay_root(plan), exist_ok=True)
    try:
        os.remove(gone_path(plan, shard))
    except FileNotFoundError:
        pass
    write_replacing(os.path.join(relay_root(plan), f"shard-{shard}.json"), json.dumps({"shard": shard, "host": host_address}))


def registrations(plan: str) -> dict[int, str]:
    """已登记的各片：片号 → 那一片所在那一台的地址。写到一半读不通的当没登记（下一轮再读）。"""
    found: dict[int, str] = {}
    root = relay_root(plan)
    for name in sorted(os.listdir(root)) if os.path.isdir(root) else []:
        if not (name.startswith("shard-") and name.endswith(".json")):
            continue
        try:
            with open(os.path.join(root, name), encoding="utf-8") as handle:
                record = json.load(handle)
        except (OSError, ValueError):
            continue
        if isinstance(record, dict) and is_host_address(record.get("host")) and isinstance(record.get("shard"), int):
            found[record["shard"]] = record["host"]
    return found


def mark_gone(plan: str, shard: int) -> None:
    """代跑那一片退出（手上各条的退出码都已写出）：之后转交来的请求没人认领，转交的一方见到这个文件就不再等。"""
    if os.path.isdir(relay_root(plan)):
        write_replacing(gone_path(plan, shard), "gone\n")


def mark_finished(plan: str) -> None:
    if os.path.isdir(relay_root(plan)):
        write_replacing(finished_path(plan), "finished\n")


class Beat:
    """这一片的心跳：后台线程每 BEAT_SECONDS 秒把计数加一写进 shard-<片号>.beat。"""

    def __init__(self, plan: str, shard: int) -> None:
        self.path = os.path.join(relay_root(plan), f"shard-{shard}.beat")
        self.count = 0
        self.stopped = threading.Event()
        self.thread = threading.Thread(target=self.loop, daemon=True)

    def write(self) -> None:
        self.count += 1
        try:
            write_replacing(self.path, f"{self.count}\n")
        except OSError:
            pass  # 共享根一时写不进：下一拍再写；一直写不进，读的一方照心跳停了办

    def loop(self) -> None:
        while not self.stopped.wait(BEAT_SECONDS):
            self.write()

    def start(self) -> "Beat":
        os.makedirs(os.path.dirname(self.path), exist_ok=True)
        self.write()
        self.thread.start()
        return self

    def stop(self) -> None:
        self.stopped.set()


class BeatWatch:
    """看另一片的心跳：读到的数变了就记下这一刻（本机单调钟）；从没读到过的不算停（那一片还没起）。"""

    def __init__(self, plan: str, shard: int) -> None:
        self.path = os.path.join(relay_root(plan), f"shard-{shard}.beat")
        self.value: str | None = None
        self.changed_at = time.monotonic()

    def seen(self) -> bool:
        try:
            with open(self.path, encoding="utf-8") as handle:
                value = handle.read().strip()
        except OSError:
            return self.value is not None
        if value and value != self.value:
            self.value, self.changed_at = value, time.monotonic()
        return self.value is not None

    def stale(self, seconds: float) -> bool:
        return self.seen() and time.monotonic() - self.changed_at > seconds


class SignalFlag:
    """TERM / INT / HUP 只记下来，由主循环收尾。"""

    def __init__(self) -> None:
        self.received: list[int] = []
        for number in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
            signal.signal(number, self.on_signal)

    def on_signal(self, number: int, frame) -> None:  # noqa: ARG002
        self.received.append(number)


# ── 代跑的一面 ──
class Served:
    def __init__(self, process: subprocess.Popen, scratch: str, directory: str) -> None:
        self.process, self.scratch, self.directory, self.stop_sent = process, scratch, directory, False


def signal_group(process: subprocess.Popen) -> None:
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass


def start_request(directory: str, scratch_base: str) -> Served | None:
    """认领一条请求并起它；已被认领或请求还没写全交 None。"""
    try:
        with open(os.path.join(directory, "request.json"), encoding="utf-8") as handle:
            request = json.load(handle)
    except (OSError, ValueError):
        return None
    try:
        os.mkdir(os.path.join(directory, "taken"))
    except FileExistsError:
        return None
    scratch = os.path.join(scratch_base, os.path.basename(directory))
    shutil.rmtree(scratch, ignore_errors=True)
    os.makedirs(scratch)
    environment = dict(os.environ, **{SCRATCH_VARIABLE: scratch, "CARGO_TARGET_DIR": os.path.join(scratch, "target")})
    with open(os.path.join(directory, "stdout"), "x", encoding="utf-8") as out, open(os.path.join(directory, "stderr"), "x", encoding="utf-8") as err:
        try:
            process = subprocess.Popen(request["argv"], cwd=request["cwd"], env=environment, stdout=out, stderr=err, start_new_session=True)
        except OSError as error:
            err.write(f"代跑那一片起不了这条命令：{error}\n")
            shutil.rmtree(scratch, ignore_errors=True)
            write_replacing(os.path.join(directory, "exit"), f"{EXIT_TRANSPORT}\n")
            return None
    return Served(process, scratch, directory)


def finish_request(item: Served, code: int) -> None:
    """命令被信号停的（Popen 交负数）照 shell 的写法记 128 + 信号号（TERM 是 143）。"""
    code = code if code >= 0 else 128 - code
    shutil.rmtree(item.scratch, ignore_errors=True)
    write_replacing(os.path.join(item.directory, "exit"), f"{code}\n")


def stop_all(running: dict[str, Served]) -> None:
    for item in running.values():
        signal_group(item.process)
    for label, item in list(running.items()):
        finish_request(item, item.process.wait())
        del running[label]


def abandon_orphans(directory: str) -> list[str]:
    """这一片上一次（服务重试之前那一次）认领了、没写退出码的请求：那条命令随上一次一起没了，记 EXIT_TRANSPORT，转交的一方照没跑成办。"""
    abandoned = []
    for label in sorted(os.listdir(directory)):
        path = os.path.join(directory, label)
        if os.path.isdir(os.path.join(path, "taken")) and not os.path.exists(os.path.join(path, "exit")):
            write_replacing(os.path.join(path, "exit"), f"{EXIT_TRANSPORT}\n")
            abandoned.append(label)
    return abandoned


def serve(plan: str, shard: int, poll_seconds: float, say=print, orchestrator: int = 0) -> int:
    """片号 shard 那一片：代跑编排那一片（片号 orchestrator）转交来的命令，直到它写了 finished、手上的都跑完（退 0）。
    收到停止信号停手上的命令、退 143；编排那一片心跳停了 STALE_SECONDS 秒，停手上的命令、退 1。"""
    flag = SignalFlag()
    own_scratch = os.environ.get(SCRATCH_VARIABLE, "")
    scratch_base = own_scratch if own_scratch and os.path.isdir(own_scratch) else tempfile.mkdtemp(prefix=f"e171-shard-{shard}-relay-")
    made_scratch = scratch_base != own_scratch
    directory = inbox(plan, shard)
    os.makedirs(directory, exist_ok=True)
    abandoned = abandon_orphans(directory)
    if abandoned:
        say(f"代跑那一片（第 {shard} 片）是重试的那一次：上一次没跑完的 {len(abandoned)} 条记退 {EXIT_TRANSPORT}（{','.join(abandoned[:4])}）")
    beat = Beat(plan, shard).start()
    watch = BeatWatch(plan, orchestrator)
    running: dict[str, Served] = {}
    served = 0
    try:
        while True:
            if flag.received:
                stop_all(running)
                say(f"代跑那一片（第 {shard} 片）收到停止信号：手上的命令已停，退 {EXIT_STOPPED_BY_SIGNAL}")
                return EXIT_STOPPED_BY_SIGNAL
            for label in sorted(os.listdir(directory)):
                path = os.path.join(directory, label)
                if label in running or os.path.exists(os.path.join(path, "exit")) or not os.path.isdir(path):
                    continue
                item = start_request(path, scratch_base)
                if item is not None:
                    running[label] = item
                    served += 1
            for label, item in list(running.items()):
                if not item.stop_sent and os.path.exists(os.path.join(item.directory, "stop")):
                    signal_group(item.process)
                    item.stop_sent = True
                code = item.process.poll()
                if code is not None:
                    finish_request(item, code)
                    del running[label]
            if os.path.exists(finished_path(plan)) and not running:
                say(f"代跑那一片（第 {shard} 片）收尾：编排那一片已收尾，这一片代跑了 {served} 条命令")
                return 0
            if watch.stale(STALE_SECONDS):
                stop_all(running)
                say(f"✗ 编排那一片（第 {orchestrator} 片）的心跳停了 {STALE_SECONDS:.0f} 秒、没写 finished：手上的命令已停（退 {EXIT_ORCHESTRATOR_GONE}）")
                return EXIT_ORCHESTRATOR_GONE
            time.sleep(poll_seconds)
    finally:
        beat.stop()
        if made_scratch:
            shutil.rmtree(scratch_base, ignore_errors=True)
        try:
            mark_gone(plan, shard)
        except OSError:
            pass  # 共享根写不进：转交的一方照心跳停了办


# ── 转交的一面 ──
def relay(plan: str, target: int, label: str, argv: list[str], cwd: str, poll_seconds: float) -> int:
    if not argv:
        print("✗ shard_relay.py relay：-- 之后没有命令（退 2）", file=sys.stderr)
        print("  → 怎么办：照用法改：shard_relay.py relay --plan <计划目录的绝对路径> --to <片号> --label <标签> [--poll <秒>] -- <命令…>", file=sys.stderr)
        return EXIT_USAGE
    directory = os.path.join(inbox(plan, target), label)
    os.makedirs(os.path.dirname(directory), exist_ok=True)
    try:
        os.mkdir(directory)
    except FileExistsError:
        print(f"✗ shard_relay.py relay：标签 {label} 在第 {target} 片已有一条请求（退 2，命令没起）", file=sys.stderr)
        print("  → 怎么办：同一片里每条命令用一个没用过的标签；那一条是上一趟留下的，等它结束、或核过没人在等再删它的请求目录", file=sys.stderr)
        return EXIT_USAGE
    flag = SignalFlag()
    write_replacing(os.path.join(directory, "request.json"), json.dumps({"argv": argv, "cwd": cwd}, ensure_ascii=False))
    watch = BeatWatch(plan, target)
    stop_written = False
    while True:
        exit_path = os.path.join(directory, "exit")
        server_gone = os.path.exists(gone_path(plan, target))  # 先看它，再看退出码：代跑那一片先写退出码、后写 gone，这个次序读不漏
        if os.path.exists(exit_path):
            code = int(open(exit_path, encoding="utf-8").read().strip())
            for name, stream in (("stdout", sys.stdout), ("stderr", sys.stderr)):
                path = os.path.join(directory, name)
                if os.path.exists(path):
                    stream.write(open(path, encoding="utf-8", errors="replace").read())
                    stream.flush()
            return code
        if flag.received and not stop_written:
            write_replacing(os.path.join(directory, "stop"), "stop\n")
            stop_written = True
        if server_gone:
            print(f"✗ 第 {target} 片（代跑那一台）已退出，标签 {label} 那条命令没有退出码、也不会再有人认领（退 {EXIT_TRANSPORT}）", file=sys.stderr)
            print("  → 怎么办：看代跑那一片的日志为什么退出，起回那一片之后重交这条命令", file=sys.stderr)
            return EXIT_TRANSPORT
        if watch.stale(STALE_SECONDS):
            print(f"✗ 第 {target} 片（代跑那一台）的心跳停了 {STALE_SECONDS:.0f} 秒，标签 {label} 那条命令结局不明（退 {EXIT_TRANSPORT}）", file=sys.stderr)
            print("  → 怎么办：看代跑那一台还活不活、那一片的日志停在哪；结局不明的这条照没跑算，起回那一片之后重交", file=sys.stderr)
            return EXIT_TRANSPORT
        time.sleep(poll_seconds)


def main(argument_list: list[str]) -> int:
    if "--" not in argument_list:
        print("✗ 用法：shard_relay.py relay --plan <计划目录> --to <片号> --label <标签> [--poll <秒>] -- <命令…>（退 2）", file=sys.stderr)
        print("  → 怎么办：在自己的参数与要代跑的命令之间写一个 --", file=sys.stderr)
        return EXIT_USAGE
    split = argument_list.index("--")
    own, command = argument_list[:split], argument_list[split + 1:]
    if own[:1] != ["relay"]:
        print(f"✗ shard_relay.py 只认 relay，给的是 {own[:1]}（退 2）", file=sys.stderr)
        print("  → 怎么办：照用法改：shard_relay.py relay --plan <计划目录的绝对路径> --to <片号> --label <标签> [--poll <秒>] -- <命令…>", file=sys.stderr)
        return EXIT_USAGE
    options: dict[str, str] = {}
    words = own[1:]
    while words:
        if words[0] not in ("--plan", "--to", "--label", "--poll") or len(words) < 2:
            print(f"✗ shard_relay.py relay 认不得的参数 {words[0]}（只认 --plan、--to、--label、--poll；退 2）", file=sys.stderr)
            print("  → 怎么办：照用法改：shard_relay.py relay --plan <计划目录的绝对路径> --to <片号> --label <标签> [--poll <秒>] -- <命令…>", file=sys.stderr)
            return EXIT_USAGE
        options[words[0]], words = words[1], words[2:]
    if not {"--plan", "--to", "--label"} <= set(options) or not options["--to"].isdigit() or not os.path.isabs(options["--plan"]):
        print("✗ shard_relay.py relay 要 --plan <绝对路径>、--to <片号>、--label <标签>（退 2）", file=sys.stderr)
        print("  → 怎么办：照用法改：shard_relay.py relay --plan <计划目录的绝对路径> --to <片号> --label <标签> [--poll <秒>] -- <命令…>", file=sys.stderr)
        return EXIT_USAGE
    return relay(options["--plan"], int(options["--to"]), options["--label"], command, os.getcwd(), float(options.get("--poll", "1")))


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
