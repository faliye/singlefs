#!/usr/bin/env python3
"""槽位攻方腿的探针（挑槽那一段）：从冻结的 merge-and-verify.sh 取 SLOT_HOLDER_PROGRAM 原文，在给定的草稿目录里起挑槽进程，量三件事：
  登记的槽数上限压不压得住已经在的槽（lowered-limit：已有 3 槽、上限 1，同时起 3 个挑槽进程）；
  上限只管新开的槽（new-slots-capped：没有槽、上限 1，同时起 3 个，对照）；
  3 槽都留着时挑槽进程答什么（all-kept：3 槽都有 kept 与 repo/、上限 3）。
用法：python3 slot_probe.py --script <冻结的 merge-and-verify.sh> --work <不在或空的草稿目录>
每格打一行 name=<格名> … verdict=<…>；退 0 跑完（打没打中看 verdict），2 用法错，1 探针自己出错。
挑槽进程各放进自己的进程组（start_new_session），每格收尾只对自己起的那几组发 TERM，再核没有一个留下。"""
import argparse
import os
import selectors
import signal
import subprocess
import sys
import tempfile
import time

ANSWER_SECONDS = 20


def holder_program(script_path):
    text = open(script_path, encoding="utf-8").read()
    marker = "\nSLOT_HOLDER_PROGRAM='\n"
    begin = text.index(marker) + len(marker)
    end = text.index("\n'\n", begin)
    return text[begin:end + 1]


def make_slots(root, count, kept):
    slots = []
    for _ in range(count):
        slot = tempfile.mkdtemp(prefix="slot.", dir=root)
        os.mkdir(os.path.join(slot, "repo"))
        with open(os.path.join(slot, "used"), "w") as stamp:
            stamp.write(f"{time.time_ns()}\n")
        if kept:
            with open(os.path.join(slot, "kept"), "w") as record:
                record.write("log: probe\ncreated: probe\n")
        slots.append(os.path.realpath(slot))
    return slots


def start_holder(program, root, limit):
    return subprocess.Popen([sys.executable, "-c", program, root, str(limit), "", str(os.getpid()), ""],
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, start_new_session=True)


def read_answer(holder):
    chooser = selectors.DefaultSelector()
    chooser.register(holder.stdout, selectors.EVENT_READ)
    ready = chooser.select(timeout=ANSWER_SECONDS)
    chooser.close()
    if not ready:
        return "NO-ANSWER"
    return holder.stdout.readline().rstrip("\n") or "EMPTY"


def stop_holders(holders):
    for holder in holders:
        if holder.poll() is None:
            os.killpg(holder.pid, signal.SIGTERM)
    for holder in holders:
        try:
            holder.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(holder.pid, signal.SIGKILL)
            holder.wait()
    return sum(1 for holder in holders if holder.poll() is None)


def run_case(program, work, name, existing, kept, limit, starts):
    root = os.path.join(work, name)
    os.makedirs(root)
    make_slots(root, existing, kept)
    holders, answers = [], []
    try:
        for _ in range(starts):
            holder = start_holder(program, root, limit)
            holders.append(holder)
            answers.append(read_answer(holder))
    finally:
        left = stop_holders(holders)
    held = sorted({answer.split("\t")[0][len("SLOT held "):] for answer in answers if answer.startswith("SLOT held ")})
    none = [answer for answer in answers if answer.startswith("SLOT none ")]
    slot_directories = sorted(entry for entry in os.listdir(root) if entry.startswith("slot."))
    return held, none, answers, slot_directories, left


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--script", required=True)
    parser.add_argument("--work", required=True)
    arguments = parser.parse_args()
    if not os.path.isfile(arguments.script):
        print(f"--script {arguments.script} 不是文件", file=sys.stderr)
        return 2
    if os.path.exists(arguments.work) and os.listdir(arguments.work):
        print(f"--work {arguments.work} 已存在且非空", file=sys.stderr)
        return 2
    os.makedirs(arguments.work, exist_ok=True)
    program = holder_program(arguments.script)
    held, none, answers, slots, left = run_case(program, arguments.work, "lowered-limit", existing=3, kept=False, limit=1, starts=3)
    for answer in answers:
        print(f"  lowered-limit 挑槽进程答：{answer}")
    print(f"name=lowered-limit existing_slots=3 limit=1 holders=3 held_distinct={len(held)} none={len(none)} slot_directories={len(slots)} holders_left={left} "
          f"verdict={'HIT-limit-does-not-bound-existing-slots' if len(held) > 1 else 'limit-bounds-existing-slots'}")
    held, none, answers, slots, left = run_case(program, arguments.work, "new-slots-capped", existing=0, kept=False, limit=1, starts=3)
    for answer in answers:
        print(f"  new-slots-capped 挑槽进程答：{answer}")
    print(f"name=new-slots-capped existing_slots=0 limit=1 holders=3 held_distinct={len(held)} none={len(none)} slot_directories={len(slots)} holders_left={left} "
          f"verdict={'limit-bounds-new-slots' if len(held) == 1 and len(slots) == 1 else 'UNEXPECTED'}")
    held, none, answers, slots, left = run_case(program, arguments.work, "all-kept", existing=3, kept=True, limit=3, starts=1)
    for answer in answers:
        print(f"  all-kept 挑槽进程答：{answer}")
    reports_kept = any("3 槽是带 MERGE_AND_VERIFY_KEEP_COPY=1 留下快照" in answer for answer in none)
    print(f"name=all-kept existing_slots=3 kept=3 limit=3 holders=1 held_distinct={len(held)} none={len(none)} holders_left={left} "
          f"verdict={'none-answer-names-kept-slots' if len(held) == 0 and reports_kept else 'UNEXPECTED'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
