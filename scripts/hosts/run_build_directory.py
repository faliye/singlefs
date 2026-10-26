#!/usr/bin/env python3
"""每一趟自己建的编译目录：跑完删、删之前核没有进程开着里面的文件。

起 cargo 的入口脚本（合入后验证、crates 变异门禁、崩溃点重放、崩溃放量、崩溃点重放分片包装、multi-host-run.sh 两侧）每一趟在自己的运行目录里建
CARGO_TARGET_DIR，正常结束、出错、被 TERM / INT 停都删掉自己建的这一份；调用方给了 CARGO_TARGET_DIR 的不删（不是入口建的，归调用方）。
不留给以后的趟复用，也不读写跨轮共用的编译缓存。

删之前核：本机此刻有没有进程（本进程除外）的工作目录、根目录、可执行文件、打开的文件描述符或映射进内存的文件落在这个目录里。
有就是起它的那一趟的子进程没收干净：不删，逐个报进程号、命令与开着的是哪一样，退 1。读不了的进程（别的用户的、刚退出的）跳过。

shell 入口经命令行调，python 入口 import 本库调 remove_run_build_directory：
  run_build_directory.py remove <目录> [--skip-open-check]
      目录不在：打一行「这一趟的编译目录 <目录> 没建」，退 0。
      有进程开着里面的文件：打「✗ …没删」与逐个进程，下一行「→ 怎么办」，退 1。
      删了、回读不在了：打一行「删了这一趟的编译目录 <目录>（共 <大小>）」，退 0；删不成（回读还在）退 1。
      --skip-open-check 只给调用方的弄坏开关用（不核开着的文件就删）。
  run_build_directory.py holders <目录>
      逐行打开着里面文件的进程，退 0 没有、3 有。
本库不读弄坏开关；调用方的弄坏开关（各自文件头开关表）决定调不调、带不带 --skip-open-check。
"""
import os
import shutil
import sys

sys.dont_write_bytecode = True

EXIT_HOLDERS = 3


def _inside(path, directory):
    if path.endswith(" (deleted)"):
        path = path[: -len(" (deleted)")]
    return path == directory or path.startswith(directory + os.sep)


def _command_of(base):
    try:
        with open(os.path.join(base, "cmdline"), "rb") as handle:
            words = [word.decode(errors="replace") for word in handle.read().split(b"\0") if word]
    except OSError:
        return "?"
    return " ".join(words)[:160] or "?"


def open_holders(directory):
    """[(进程号, 命令, 开着的是哪一样)]：本机此刻开着 directory 里文件的进程（本进程除外）。目录不在交空表。"""
    if not os.path.isdir(directory):
        return []
    real = os.path.realpath(directory)
    holders = []
    own = os.getpid()
    for entry in os.listdir("/proc"):
        if not entry.isdigit() or int(entry) == own:
            continue
        base = os.path.join("/proc", entry)
        found = []
        for link, what in (("cwd", "工作目录"), ("root", "根目录"), ("exe", "可执行文件")):
            try:
                if _inside(os.readlink(os.path.join(base, link)), real):
                    found.append(what)
            except OSError:
                pass
        try:
            descriptors = os.listdir(os.path.join(base, "fd"))
        except OSError:
            descriptors = []
        opened = 0
        for descriptor in descriptors:
            try:
                if _inside(os.readlink(os.path.join(base, "fd", descriptor)), real):
                    opened += 1
            except OSError:
                pass
        if opened:
            found.append(f"{opened} 个文件描述符")
        try:
            with open(os.path.join(base, "maps"), encoding="utf-8", errors="replace") as handle:
                mapped = sum(1 for line in handle if len(line.split(None, 5)) == 6 and _inside(line.split(None, 5)[5].strip(), real))
        except OSError:
            mapped = 0
        if mapped:
            found.append(f"{mapped} 段内存映射")
        if found:
            holders.append((int(entry), _command_of(base), "、".join(found)))
    return holders


def allocated_size_text(directory):
    total = 0
    for current, subdirectories, files in os.walk(directory):
        for name in (*files, *subdirectories):
            try:
                total += os.lstat(os.path.join(current, name)).st_blocks * 512
            except OSError:
                pass
    for unit, size in (("GiB", 1 << 30), ("MiB", 1 << 20), ("KiB", 1 << 10)):
        if total >= size:
            return f"{total / size:.1f} {unit}"
    return f"{total} B"


def remove_run_build_directory(directory, check_open=True):
    """删这一趟自己建的编译目录。回 (删成了没有, [要打的行])；目录不在算删成。"""
    directory = os.path.abspath(directory)
    if not os.path.lexists(directory):
        return True, [f"  这一趟的编译目录 {directory} 没建，不用删"]
    holders = open_holders(directory) if check_open else []
    if holders:
        lines = [f"  ✗ 这一趟的编译目录 {directory} 还有 {len(holders)} 个进程开着里面的文件，没删（起它的那一趟的子进程没收干净）："]
        lines += [f"      进程 {pid}（{command}）开着{what}" for pid, command, what in holders]
        lines.append(f"  → 怎么办：逐个 ps -o pid,ppid,args -p <进程号> 看是哪一步漏下的，python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号> 停掉，"
                     f"再 python3 scripts/hosts/run_build_directory.py remove {directory}；修起它的脚本，让它停自己的子进程之后再删")
        return False, lines
    size = allocated_size_text(directory)
    shutil.rmtree(directory, ignore_errors=True)
    if os.path.lexists(directory):
        return False, [f"  ✗ 这一趟的编译目录 {directory} 删不成（rmtree 之后回读还在）",
                       f"  → 怎么办：ls -la {directory} 看剩下什么、权限对不对，核没有进程开着之后手删"]
    return True, [f"  删了这一趟的编译目录 {directory}（共 {size}）"]


def main(arguments):
    if len(arguments) >= 2 and arguments[0] == "remove" and set(arguments[2:]) <= {"--skip-open-check"}:
        ok, lines = remove_run_build_directory(arguments[1], check_open="--skip-open-check" not in arguments[2:])
        print("\n".join(lines))
        return 0 if ok else 1
    if len(arguments) == 2 and arguments[0] == "holders":
        holders = open_holders(os.path.abspath(arguments[1]))
        for pid, command, what in holders:
            print(f"进程 {pid}（{command}）开着{what}")
        return EXIT_HOLDERS if holders else 0
    print("  ✗ 用法不对：run_build_directory.py remove <目录> [--skip-open-check] | holders <目录>")
    print("  → 怎么办：照文件头的用法写；删的是调用方这一趟自己建的编译目录，调用方给的 CARGO_TARGET_DIR 不交给它")
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
