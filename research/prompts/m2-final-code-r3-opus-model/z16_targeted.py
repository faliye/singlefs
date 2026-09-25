#!/usr/bin/env python3
"""Z16 定点扫描：每个（单元区宽, 覆盖写次数）格先跑一次对照，取回退那次挂载写行发布释放的槽；
再对每个槽在盘 1 上注入「读一律报错」各跑一次（回退之前开），看预演与真发分不分叉、真发会不会被拒。8 路并行。
用法：z16_targeted.py <二进制> <输出 tsv> [宽起 宽止 步] [覆盖写起 止]"""
import os, re, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

BIN, OUT = sys.argv[1], sys.argv[2]
w0, w1, ws = (int(x) for x in (sys.argv[3:6] or [240, 384, 4]))
n0, n1 = (int(x) for x in (sys.argv[6:8] or [18, 35]))

def run(width, n, test, extra):
    env = dict(os.environ, SINGLEFS_R3_OPUS_UNIT_AREA_SLOTS=str(width), R3_OPUS_OVERWRITES=str(n),
               SINGLEFS_R3_OPUS_REPORT="1", **extra)
    out = subprocess.run(["nice", "-n", "19", BIN, "--exact", test, "--nocapture"], env=env,
                         capture_output=True, text=True).stderr
    return [l for l in out.splitlines() if l.startswith("R3OPUS")] + \
           [l for l in out.splitlines() if "panicked" in l]

def summarize(lines):
    dry = [l for l in lines if l.startswith("R3OPUS-DRYRUN")]
    runl = [l for l in lines if l.startswith("R3OPUS-RUN")]
    rb = runl[-1] if runl else "NO-RUN " + " ".join(lines)[:300]
    m = re.search(r"ending=(\S+)", rb); ending = m.group(1)[:120] if m else "?"
    m = re.search(r"rollback=Some\((\w+)(?:\((\w+)| \{ member: \"([^\"]*)\")?", rb)
    rbk = (m.group(1) + ":" + (m.group(2) or m.group(3) or "")) if m else "?"
    m = re.search(r"rewritten_from_released: (\d+)", rb.split("rollback=")[-1].split("after=")[0])
    reuse = m.group(1) if m else "-"
    last_dry = dry[-1] if len(dry) >= 2 else ""
    q = re.search(r"quarantined=(\w+) same=(\w+)", last_dry)
    return ending, rbk, reuse, (q.group(1) if q else "-"), (q.group(2) if q else "-"), last_dry

cells = [(w, n) for w in range(w0, w1 + 1, ws) for n in range(n0, n1 + 1)]
def control(cell):
    w, n = cell
    lines = run(w, n, "z16_control", {})
    rel = [l for l in lines if l.startswith("R3OPUS-RELEASED")]
    s = summarize(lines)
    slots = []
    if len(rel) >= 2 and s[1].startswith("Applied"):
        first = re.search(r"\[\[([0-9, ]*)\]", rel[-1])
        if first:
            slots = [int(x) for x in first.group(1).split(",") if x.strip()]
    return (w, n, s, slots)

with ThreadPoolExecutor(8) as pool:
    controls = list(pool.map(control, cells))
jobs = [(w, n, slot) for (w, n, s, slots) in controls for slot in slots]
def faulted(job):
    w, n, slot = job
    lines = run(w, n, "z16_sweep_read_fault_on_one_slot",
                {"R3_OPUS_DEVICE": os.environ.get("R3_OPUS_FAULT_DEVICE", "1"), "R3_OPUS_SLOT_FROM": str(slot), "R3_OPUS_SLOT_TO": str(slot + 1)})
    return (w, n, slot, summarize(lines))
with ThreadPoolExecutor(8) as pool:
    results = list(pool.map(faulted, jobs))
with open(OUT, "w") as f:
    f.write("width\toverwrites\tfault_slot\tending\trollback\treuse\tquarantined\tsame\tdryrun_line\n")
    for (w, n, s, slots) in controls:
        f.write(f"{w}\t{n}\tnone\t{s[0]}\t{s[1]}\t{s[2]}\t{s[3]}\t{s[4]}\t{s[5][:600]}\n")
    for (w, n, slot, s) in results:
        f.write(f"{w}\t{n}\t{slot}\t{s[0]}\t{s[1]}\t{s[2]}\t{s[3]}\t{s[4]}\t{s[5][:600]}\n")
print(len(controls), "controls;", len(results), "faulted runs")
