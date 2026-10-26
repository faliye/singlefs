#!/usr/bin/env python3
"""V1 探针：把 run-with-memory-cap.sh 里 admission() 的内嵌 python（到 operation 分派之前）原样载入，
往 HostLookup 的 begin / finish 注入「起不了线程」「临时文件建不了」，看异常往不往主线程外冒。
用法：exception_probe.py <run-with-memory-cap.sh 的路径> <草稿目录>
注入的是条件，不是真把机器资源用光：自然触发要不要、多容易出现，这里没量。"""
import os
import sys
import tempfile
import threading

script_path, work = sys.argv[1], sys.argv[2]
body = open(script_path, encoding="utf-8").read()
start = body.index("3<<'PY'\n", body.index("admission() {")) + len("3<<'PY'\n")
head = body[start:body.index("\nPY\n", start)]
head = head[:head.index("operation, arguments = sys.argv[1], sys.argv[2:]")]
state = os.path.join(work, "state")
os.makedirs(state, exist_ok=True)
os.environ.update(RUN_WITH_MEMORY_CAP_STATE_DIR_RESOLVED=state, RUN_WITH_MEMORY_CAP_PEAKS_RESOLVED=os.path.join(state, "peaks.tsv"),
                  RUN_WITH_MEMORY_CAP_SCRIPT_DIRECTORY_RESOLVED=os.path.dirname(os.path.abspath(script_path)),
                  RUN_WITH_MEMORY_CAP_WAIT_SECONDS_RESOLVED="1", RUN_WITH_MEMORY_CAP_THREADS_RESOLVED="")
os.environ.pop("RUN_WITH_MEMORY_CAP_HOST_LOOKUP", None)
namespace = {"__name__": "admission_probe"}
exec(compile(head, "admission_head.py", "exec"), namespace)
HostLookup = namespace["HostLookup"]


def trial(name, patch, call):
    lookup = HostLookup("unit-probe", 1 << 30, "1G", "probe-key", os.getpid(), ["true"])
    undo = patch()
    try:
        call(lookup)
        print(f"{name}: 没有异常冒出来")
    except BaseException as error:
        print(f"{name}: 异常冒出来 {type(error).__name__}: {error}")
    finally:
        undo()


def patch_thread_start():
    saved = threading.Thread.start

    def refuse(self):
        raise RuntimeError("can't start new thread")
    threading.Thread.start = refuse
    return lambda: setattr(threading.Thread, "start", saved)


def patch_temporary_file():
    saved = tempfile.TemporaryFile

    def refuse(*args, **keywords):
        raise OSError(28, "No space left on device")
    namespace["tempfile"].TemporaryFile = refuse
    return lambda: setattr(namespace["tempfile"], "TemporaryFile", saved)


trial("begin（起线程失败，主线程里）", patch_thread_start, lambda lookup: lookup.begin({"need": 1}))
trial("finish（临时文件建不了，退 252 之前前台查，主线程里）", patch_temporary_file, lambda lookup: lookup.finish({"need": 1}))
trial("begin（临时文件建不了，后台线程里查）", patch_temporary_file, lambda lookup: (lookup.begin({"need": 1}), lookup.thread.join()))
