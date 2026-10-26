#!/usr/bin/env python3
"""hostlookup_inject.py <run-with-memory-cap.sh>：把包装内嵌 python 里的 class HostLookup 原文摘出来，在假的全局（假 host-resources.py、假状态目录）里直接调它的 begin / finish / cancel，
各注入一个故障，看异常有没有冒到调用方（主线程）。不起 systemd、不起真的包装。
故障：① 起线程失败；② 建临时文件失败（tempfile.tempdir 指到不存在的目录）；③ 读输出之后格式化抛 TypeError（假 human）；
      ④ 排队锁（admission_lock）一直不放；⑤ 等线程的 join 不带时限（线程里卡住）。
每一项打一行：name=inject point=<点> where=<调用> escaped=<True|False> exception=<类型>"""
import contextlib, datetime, json, os, re, shlex, subprocess, sys, tempfile, threading, time, types
wrapper = sys.argv[1]
text = open(wrapper, encoding="utf-8").read()
start = text.index("class HostLookup:")
end = text.index("\ndef admit(", start)
source = text[start:end]
scratch = tempfile.mkdtemp(prefix="hostlookup-inject-")
fake_dir = os.path.join(scratch, "scripts-hosts"); os.makedirs(fake_dir)
open(os.path.join(fake_dir, "host-resources.py"), "w").write(
    "import json, sys\n"
    "print(json.dumps({'local_only_because': '', 'hosts': [{'is_peer': True, 'name': '第二台', 'memory': {'available': 40 << 30}, 'memory_cap': None}],"
    " 'command': {'prediction': {'memory': 8 << 30}, 'verdicts': [{'name': '第二台', 'read': True, 'fits': True, 'why': ''}]}}))\n")
os.environ["RUN_WITH_MEMORY_CAP_SCRIPT_DIRECTORY_RESOLVED"] = fake_dir
for name in ("RUN_WITH_MEMORY_CAP_HOST_LOOKUP", "RUN_WITH_MEMORY_CAP_HOST_LOOKUP_SECONDS"):
    os.environ.pop(name, None)
lock_state = {"hang": False}
@contextlib.contextmanager
def admission_lock():
    if lock_state["hang"]:
        time.sleep(3600)
    yield
def build(human=None):
    names = {"broken": "", "state_directory": scratch, "admission_lock": admission_lock, "HOST_LOOKUP_SECONDS": 20.0, "HOST_LOOKUP_EVERY_SECONDS": 0.0,
             "HOST_LOOKUP_PEER_TIMEOUT_SECONDS": 8, "HOST_LOOKUP_SAMPLE_SECONDS": 0.2, "say": lambda line: None,
             "human": human or (lambda n: str(n)), "size_bytes": lambda s: None, "alerts_record_path": lambda: None,
             "MEMORY_QUEUE_RECORD_KIND": "k", "datetime": datetime, "ZoneInfo": lambda n: datetime.timezone.utc,
             "json": json, "os": os, "shlex": shlex, "subprocess": subprocess, "sys": sys, "tempfile": tempfile, "threading": threading, "time": time, "contextlib": contextlib}
    namespace = dict(names)
    exec(source, namespace)
    return namespace["HostLookup"]("unit", 1 << 30, "1G", "key", os.getpid(), ["true"])
def attempt(point, where, action):
    try:
        action()
        print(f"name=inject point={point} where={where} escaped=False exception=-")
    except BaseException as error:
        print(f"name=inject point={point} where={where} escaped=True exception={type(error).__name__}")
# ① 起线程失败
real_thread = threading.Thread
class FailingThread:
    def __init__(self, *a, **k): pass
    def start(self): raise RuntimeError("can't start new thread")
threading.Thread = FailingThread
attempt("起线程失败", "begin()", lambda: build().begin({}))
threading.Thread = real_thread
# ② 建临时文件失败
tempfile.tempdir = os.path.join(scratch, "no-such-dir")
attempt("建临时文件失败", "finish()（退 252 之前，主线程）", lambda: build().finish({}))
tempfile.tempdir = None
# ③ 读输出之后格式化抛 TypeError
def bad_human(_value): raise TypeError("injected")
attempt("格式化抛 TypeError", "finish()（退 252 之前，主线程）", lambda: build(bad_human).finish({}))
# ④ 排队锁一直不放：finish() 里的 take_turn 取锁
lock_state["hang"] = True
holder = real_thread(target=lambda: build().finish({}), daemon=True)
started = time.monotonic(); holder.start(); holder.join(timeout=3)
print(f"name=inject point=排队锁不放 where=finish() escaped={holder.is_alive()} exception=卡住（3 秒后调用还没返回）" if holder.is_alive() else "name=inject point=排队锁不放 where=finish() escaped=False exception=-")
lock_state["hang"] = False
# ⑤ join 不带时限：线程里 query 卡住时 finish() 的 self.thread.join()
class Sleepy:
    def join(self): time.sleep(3600)
lookup = build(); lookup.thread = Sleepy()
holder = real_thread(target=lambda: lookup.finish({}), daemon=True)
holder.start(); holder.join(timeout=3)
print(f"name=inject point=等线程的 join 无时限 where=finish() escaped={holder.is_alive()} exception=卡住（3 秒后调用还没返回）")
