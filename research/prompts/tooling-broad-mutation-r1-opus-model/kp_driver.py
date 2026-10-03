#!/usr/bin/env python3
"""把副本驱动当模块载入，在 KP 点名的那一处给自己发 SIGKILL，然后照常调它的 main。只在草稿目录的假库上用。
用法：KP=<点名> KP_SEQ=<序号> python3 kp_driver.py <副本驱动路径> <驱动参数…>"""
import importlib.util, os, signal, sys

path, argv = sys.argv[1], sys.argv[2:]
spec = importlib.util.spec_from_file_location("__broad_copy__", path)
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
KP, SEQ = os.environ.get("KP", ""), os.environ.get("KP_SEQ", "")
current = {"sequence": None, "armed": False}


def die():
    os.kill(os.getpid(), signal.SIGKILL)


real_apply, real_atomic, real_record = m.apply_mutation, m.write_atomically, m.Library.record_result
real_replace, real_remove, real_fsync = os.replace, os.remove, os.fsync


def apply_mutation(library, item):
    current["sequence"] = str(item["sequence"])
    real_apply(library, item)
    if KP == "apply:after-mutate" and current["sequence"] == SEQ:
        die()


def write_atomically(target, data):
    inflight = target.endswith("inflight.json") and current["sequence"] == SEQ
    if KP == "apply:after-original" and inflight:
        die()
    real_atomic(target, data)
    if KP == "apply:after-record" and inflight:
        die()


def record_result(self, fields):
    hit = str(fields[0]) == SEQ and os.environ.get("KP_OUTCOME", fields[3]) == fields[3]
    if KP == "record:before" and hit:
        die()
    if KP == "record:between-write-fsync" and hit:
        current["armed"] = True
    real_record(self, fields)
    if KP == "record:after" and hit:
        die()


def fsync(descriptor):
    if current["armed"]:
        die()
    return real_fsync(descriptor)


def replace(source, target):
    restoring = ".restore." in str(source)
    if KP == "restore:before-replace" and restoring and (current["sequence"] or "startup") == SEQ:
        die()
    if KP == "sync:before-swap-in" and str(source).endswith("tree.new"):
        die()
    real_replace(source, target)
    if KP == "restore:after-replace" and restoring and current["sequence"] == SEQ:
        die()
    if KP == "sync:after-swap-in" and str(source).endswith("tree.new"):
        die()


def remove(target):
    real_remove(target)
    if KP == "restore:after-remove-record" and str(target).endswith("inflight.json") and current["sequence"] == SEQ:
        die()


m.apply_mutation, m.write_atomically, m.Library.record_result = apply_mutation, write_atomically, record_result
os.replace, os.remove, os.fsync = replace, remove, fsync
try:
    sys.exit(m.main(argv))
except m.Refusal as refusal:
    m.print_refusal(refusal)
    sys.exit(refusal.code)
except m.Stopped:
    sys.exit(m.EXIT_STOPPED)
