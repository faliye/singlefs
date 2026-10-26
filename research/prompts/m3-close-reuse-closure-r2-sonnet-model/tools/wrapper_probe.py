#!/usr/bin/env python3
"""W1 探针：把 run-with-memory-cap.sh 里 admission() 内嵌 python（到 operation 分派之前）载入，对 HostLookup 注入故障、抓它打的行。
用法：wrapper_probe.py <脚本> <草稿目录> <faults|lines|join>
  faults  在 begin / finish / run / cancel 注入故障（起不了线程、建不了临时文件、第二台那一份读出来是怪值、human 抛），看异常往不往主线程外冒
  lines   四种查法结局各打哪几行，每行标是否以 run-with-memory-cap: 起头；后台线程里的异常也抓
  join    后台线程卡在排队锁上时 finish 要不要等到天荒地老（子进程跑，12 秒收）
注入的是条件，不是真把机器资源用光。"""
import contextlib
import io
import os
import subprocess
import sys
import tempfile
import threading
import time

script_path, work, mode = sys.argv[1], sys.argv[2], sys.argv[3]


def load_head(path, workdir):
    body = open(path, encoding="utf-8").read()
    start = body.index("3<<'PY'\n", body.index("admission() {")) + len("3<<'PY'\n")
    head = body[start:body.index("\nPY\n", start)]
    head = head[:head.index("operation, arguments = sys.argv[1], sys.argv[2:]")]
    state = os.path.join(workdir, "state")
    os.makedirs(state, exist_ok=True)
    os.environ.update(RUN_WITH_MEMORY_CAP_STATE_DIR_RESOLVED=state, RUN_WITH_MEMORY_CAP_PEAKS_RESOLVED=os.path.join(state, "peaks.tsv"),
                      RUN_WITH_MEMORY_CAP_SCRIPT_DIRECTORY_RESOLVED=os.path.dirname(os.path.abspath(path)),
                      RUN_WITH_MEMORY_CAP_WAIT_SECONDS_RESOLVED="1", RUN_WITH_MEMORY_CAP_THREADS_RESOLVED="",
                      SINGLEFS_CORE_USAGE_ALERTS=os.path.join(workdir, "alerts.jsonl"))
    os.environ.pop("RUN_WITH_MEMORY_CAP_HOST_LOOKUP", None)
    namespace = {"__name__": "admission_probe"}
    exec(compile(head, "admission_head.py", "exec"), namespace)
    return namespace


PEER_FITS = {"name": "第二台", "available": 43 * 1024 ** 3, "need": 8 * 1024 ** 3, "fits": True, "memory_cap": "40G", "why": ""}


def lookup_with(namespace, query_result):
    lookup = namespace["HostLookup"]("unit-probe", 1 << 30, "1G", "probe-key", os.getpid(), ["true"])
    lookup.query = lambda: query_result
    return lookup


def capture(call):
    buffer = io.StringIO()
    escaped = None
    with contextlib.redirect_stderr(buffer):
        try:
            call()
        except BaseException as error:
            escaped = f"{type(error).__name__}: {error}"
    return escaped, buffer.getvalue()


if mode == "faults":
    namespace = load_head(script_path, work)
    results = []

    def cell(name, build, call):
        lookup, undo = build()
        escaped, text = capture(lambda: call(lookup))
        undo()
        lines = [line for line in text.splitlines() if line.strip()]
        print(f"CELL {name}: " + (f"异常冒出来 {escaped}" if escaped else "没有异常冒出来") + (f"；打了 {len(lines)} 行" if lines else ""))

    def thread_start_refused():
        saved = threading.Thread.start

        def refuse(self):
            raise RuntimeError("can't start new thread")
        threading.Thread.start = refuse
        return lookup_with(namespace, ("", PEER_FITS)), lambda: setattr(threading.Thread, "start", saved)

    def tempfile_refused():
        saved = namespace["tempfile"].TemporaryFile

        def refuse(*args, **keywords):
            raise OSError(28, "No space left on device")
        namespace["tempfile"].TemporaryFile = refuse
        lookup = namespace["HostLookup"]("unit-probe", 1 << 30, "1G", "probe-key", os.getpid(), ["true"])
        return lookup, lambda: setattr(namespace["tempfile"], "TemporaryFile", saved)

    def odd_peer_memory_cap():   # query 读出来的第二台那一份里 memory_cap 是个数（host-resources.py 以后换了写法）
        return lookup_with(namespace, ("", dict(PEER_FITS, memory_cap=5))), lambda: None

    def human_raises():
        saved = namespace["human"]
        namespace["human"] = lambda value: (_ for _ in ()).throw(ValueError("human 读不了"))
        return lookup_with(namespace, ("", PEER_FITS)), lambda: namespace.__setitem__("human", saved)

    def cancel_with_dead_process():
        lookup = lookup_with(namespace, ("", PEER_FITS))

        class Broken:
            def poll(self):
                raise OSError("poll 读不了")
        lookup.process = Broken()
        return lookup, lambda: None

    cell("begin：起不了线程", thread_start_refused, lambda lookup: lookup.begin({"need": 1}))
    cell("finish：退 252 之前前台建不了临时文件", tempfile_refused, lambda lookup: lookup.finish({"need": 1}))
    cell("finish：第二台那一份的 memory_cap 不是字符串", odd_peer_memory_cap, lambda lookup: lookup.finish({"need": 1}))
    cell("finish：human 抛 ValueError", human_raises, lambda lookup: lookup.finish({"need": 1}))
    cell("cancel：process.poll 抛 OSError", cancel_with_dead_process, lambda lookup: lookup.cancel(admitted=True))
    cell("begin：后台线程里建不了临时文件", tempfile_refused, lambda lookup: (lookup.begin({"need": 1}), lookup.thread.join()))

elif mode == "lines":
    namespace = load_head(script_path, work)
    shapes = {
        "放得下": ("", PEER_FITS),
        "放不下": ("", dict(PEER_FITS, fits=False, why="可用 1.0 GiB，预测 8.0 GiB 放不下")),
        "没查到（带第二台 stderr 末行）": ("host-resources.py 退 3：ssh: connect to host 第二台 port 22: Connection refused", None),
    }
    for name, result in shapes.items():
        lookup = lookup_with(namespace, result)
        escaped, text = capture(lambda: lookup.run("queued", {"need": 1}))
        for line in text.splitlines():
            print(f"LINE [{name}] prefixed={'yes' if line.startswith('run-with-memory-cap:') else 'NO '} {line[:110]}")
    # 告警记录写不进
    os.environ["SINGLEFS_CORE_USAGE_ALERTS"] = "/proc/nonexistent/alerts.jsonl"
    lookup = lookup_with(namespace, ("", PEER_FITS))
    escaped, text = capture(lambda: lookup.run("queued", {"need": 1}))
    for line in text.splitlines():
        print(f"LINE [放得下而告警记录写不进] prefixed={'yes' if line.startswith('run-with-memory-cap:') else 'NO '} {line[:110]}")
    # 后台线程里的异常（线程里的 run 没兜住）：threading 的默认钩子往 stderr 打整段 traceback
    saved = namespace["tempfile"].TemporaryFile
    namespace["tempfile"].TemporaryFile = lambda *a, **k: (_ for _ in ()).throw(OSError(28, "No space left on device"))
    lookup = namespace["HostLookup"]("unit-probe", 1 << 30, "1G", "probe-key", os.getpid(), ["true"])
    escaped, text = capture(lambda: (lookup.begin({"need": 1}), lookup.thread.join()))
    namespace["tempfile"].TemporaryFile = saved
    for line in text.splitlines():
        print(f"LINE [后台线程里异常没兜住] prefixed={'yes' if line.startswith('run-with-memory-cap:') else 'NO '} {line[:110]}")

elif mode == "join":
    # 子进程里：另一个进程持排队锁 20 秒；这一边 begin（线程卡在 take_turn 的 flock 上）再 finish，看 finish 多久回来
    child = r'''
import fcntl, os, sys, time
sys.argv = ["x", sys.argv[1], sys.argv[2], "join-inner"]
'''
    if len(sys.argv) > 4 and sys.argv[4] == "inner":
        namespace = load_head(script_path, work)
        lock_path = namespace["lock_path"]
        holder = subprocess.Popen([sys.executable, "-c", f"import fcntl,time; h=open({lock_path!r},'a'); fcntl.flock(h, fcntl.LOCK_EX); time.sleep(40)"])
        time.sleep(0.5)
        lookup = namespace["HostLookup"]("unit-probe", 1 << 30, "1G", "probe-key", os.getpid(), ["true"])
        os.environ["SINGLEFS_MULTI_HOST_CONFIG"] = os.path.join(work, "no-such-multi-host.env")
        namespace["HOST_LOOKUP_SECONDS"], namespace["HOST_LOOKUP_PEER_TIMEOUT_SECONDS"] = 1.0, 1
        lookup.begin({"need": 1})
        time.sleep(0.5)
        started = time.monotonic()
        lookup.finish({"need": 1})
        print(f"JOIN finish 在 12 秒之内回来了（用了 {'不到 5' if time.monotonic() - started < 5 else '5 到 12'} 秒）")
        holder.kill()
        holder.wait()
    else:
        started = time.monotonic()
        try:
            done = subprocess.run([sys.executable, __file__, script_path, work, "join", "inner"], capture_output=True, text=True, timeout=12)
            print(done.stdout.strip() or done.stderr.strip()[-300:])
        except subprocess.TimeoutExpired:
            print(f"JOIN finish 12 秒还没回来（排队锁被别的进程持着 40 秒；线程卡在 take_turn 的 flock 上，join 没有时限）")
