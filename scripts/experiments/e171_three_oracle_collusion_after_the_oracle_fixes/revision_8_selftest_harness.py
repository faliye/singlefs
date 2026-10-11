"""E171 第 1 次重跑第一段·修订 8 那一趟的几份装置（第 0 步判法、占位核对、单元编排、逐行编排外层、两趟对照）共用的自证外壳。

每份脚本的 `--selftest` 把自己的格（名字 → 函数，函数收一个临时目录、交回 (过没过, 细节)）交给 run_selftest：
  - 自证进程把自己设成子进程的收养者（PR_SET_CHILD_SUBREAPER），每一格之后收掉它起的、还活着的每个进程（先 TERM、再 KILL），
    末尾再数一遍自己的进程树，记一格 no-leftover-processes；
  - 临时目录（也交给子进程当 TMPDIR）末尾删掉，记一格 temporary-directory-removed；
  - 不点名格时，末尾逐个打开弄坏开关（脚本自己的开关表：开关 → 该红的格），在子进程里只跑那一格，核它红在那一格；
  - 末行「自证 N 项，红 M」。
弄坏开关只在自证起的子进程里认：脚本的 break 变量在 selftest 变量不是 1 时设了，refuse_break_outside_selftest 交回一句拒因（调用方退 2）。
本外壳自己的两个开关（每份脚本的开关表里都带上）：
  sweep-skipped            每一格之后与末尾不收进程                  no-leftover-processes
  temporary-kept           末尾不删临时目录                          temporary-directory-removed
"""
from __future__ import annotations

import ctypes
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time

PR_SET_CHILD_SUBREAPER = 36
HARNESS_SWITCHES = {"sweep-skipped": "no-leftover-processes", "temporary-kept": "temporary-directory-removed"}
SWEEP_GRACE_SECONDS = 2.0


def broken(break_variable: str, item: str) -> bool:
    return item in os.environ.get(break_variable, "").split(",")


def refuse_break_outside_selftest(break_variable: str, selftest_variable: str) -> str | None:
    if os.environ.get(break_variable) and os.environ.get(selftest_variable) != "1":
        return f"✗ {break_variable} 只在 --selftest 起的子进程里认；平常跑时设了它就拒（退 2）"
    return None


def become_subreaper() -> None:
    libc = ctypes.CDLL(None, use_errno=True)
    if libc.prctl(PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) != 0:
        raise OSError(ctypes.get_errno(), "prctl(PR_SET_CHILD_SUBREAPER) 没成")


def parent_table() -> dict[int, int]:
    table: dict[int, int] = {}
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            with open(f"/proc/{entry}/stat", encoding="utf-8", errors="replace") as handle:
                text = handle.read()
        except OSError:
            continue
        fields = text[text.rfind(")") + 2:].split()
        if fields and fields[0] != "Z":
            table[int(entry)] = int(fields[1])
    return table


def live_descendants(root: int) -> list[int]:
    table = parent_table()
    found, frontier = [], [root]
    while frontier:
        parent = frontier.pop()
        children = [pid for pid, ppid in table.items() if ppid == parent]
        found += children
        frontier += children
    return found


def reap_zombies() -> None:
    while True:
        try:
            pid, _ = os.waitpid(-1, os.WNOHANG)
        except ChildProcessError:
            return
        if pid == 0:
            return


def sweep_own_processes(break_variable: str) -> list[int]:
    """收掉自己进程树里还活着的进程，交回收之前看到的那几个进程号（自证要它为空）。"""
    reap_zombies()
    found = live_descendants(os.getpid())
    if broken(break_variable, "sweep-skipped"):
        return found
    for pid in found:
        try:
            os.kill(pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
    deadline = time.monotonic() + SWEEP_GRACE_SECONDS
    while time.monotonic() < deadline and live_descendants(os.getpid()):
        reap_zombies()
        time.sleep(0.05)
    for pid in live_descendants(os.getpid()):
        try:
            os.kill(pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    time.sleep(0.05)
    reap_zombies()
    return found


def start_straggler(marker: str) -> None:
    """给 no-leftover-processes 一个已知答案：起一个不等它的子孙进程（双 fork 之后孤儿归收养者），格末必须被收掉。"""
    # 它的标准输入输出接 /dev/null：弄坏开关那一轮不收它时，它不能攥着上一层 capture_output 的管子让上一层等它 300 秒
    subprocess.Popen(["bash", "-c", f"sleep 300 </dev/null >/dev/null 2>&1 & echo $! > {marker}"], start_new_session=True,
                     stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).wait()


def run_selftest(script: str, argument_list: list[str], cells: dict, switches: dict[str, str], break_variable: str,
                 selftest_variable: str) -> int:
    """cells：格名 → 函数(临时目录) -> (过没过, 细节)。switches：脚本自己的弄坏开关 → 该红的格（外壳两个开关自动加上）。"""
    wanted = set(argument_list[argument_list.index("--cells") + 1].split(",")) if "--cells" in argument_list else None
    rounds = int(argument_list[argument_list.index("--rounds") + 1]) if "--rounds" in argument_list else 1
    every_switch = dict(switches, **HARNESS_SWITCHES)
    unknown = sorted((wanted or set()) - set(cells) - {"no-leftover-processes", "temporary-directory-removed"})
    if unknown:
        print(f"认不得的格：{','.join(unknown)}；认得的：{','.join(sorted(cells))}", file=sys.stderr)
        return 2
    become_subreaper()
    red = counted = 0
    for round_number in range(1, rounds + 1):
        scratch = tempfile.mkdtemp(prefix="e171-r8-selftest-")
        os.environ["TMPDIR"] = scratch
        results = []
        for name, function in cells.items():
            if wanted is not None and name not in wanted:
                continue
            cell_directory = os.path.join(scratch, name)
            os.makedirs(cell_directory)
            try:
                ok, detail = function(cell_directory)
            except Exception as error:  # noqa: BLE001 —— 一格炸了记红、接着跑别的格
                ok, detail = False, f"炸了：{type(error).__name__}: {error}"
            results.append((name, ok, detail))
            sweep_own_processes(break_variable)
        if wanted is None or "no-leftover-processes" in wanted:
            marker = os.path.join(scratch, "straggler.pid")
            start_straggler(marker)
            seen = sweep_own_processes(break_variable)
            leftover = live_descendants(os.getpid())
            straggler = int(open(marker, encoding="utf-8").read().strip()) if os.path.exists(marker) else -1
            results.append(("no-leftover-processes", straggler in seen and not leftover,
                            f"收之前看到 {len(seen)} 个（含故意留下的 {straggler}），收之后遗留 {len(leftover)} 个"))
        if not broken(break_variable, "temporary-kept"):
            shutil.rmtree(scratch, ignore_errors=True)
        if wanted is None or "temporary-directory-removed" in wanted:
            results.append(("temporary-directory-removed", not os.path.exists(scratch), f"临时目录 {'已删' if not os.path.exists(scratch) else '还在'}"))
        if broken(break_variable, "temporary-kept"):
            shutil.rmtree(scratch, ignore_errors=True)
        for name, ok, detail in results:
            counted += 1
            red += 0 if ok else 1
            print(f"{'✓' if ok else '✗'} 自证 {name}：{detail}", flush=True)
        if rounds > 1:
            print(f"第 {round_number} 轮：{len(results)} 格，红 {sum(1 for _, ok, _ in results if not ok)}", flush=True)
    if wanted is None and not os.environ.get(break_variable):
        for switch, cell in every_switch.items():
            completed = subprocess.run([sys.executable, "-B", script, "--selftest", "--cells", cell], capture_output=True, text=True,
                                       env=dict(os.environ, **{break_variable: switch, selftest_variable: "1"}), timeout=900)
            caught = completed.returncode == 1 and f"✗ 自证 {cell}：" in completed.stdout
            counted += 1
            red += 0 if caught else 1
            print(f"{'✓' if caught else '✗'} 弄坏开关 {switch} → 红在 {cell}：{'红了' if caught else '没红（退 ' + str(completed.returncode) + '）'}", flush=True)
        sweep_own_processes(break_variable)
    print(f"自证 {counted} 项，红 {red}")
    return 1 if red else 0
