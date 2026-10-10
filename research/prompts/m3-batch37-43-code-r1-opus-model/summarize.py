#!/usr/bin/env python3
"""把探针与入库用例的日志收成汇总（排好序、不带耗时）。用法：summarize.py <日志…>（日志名形如 <阶段>.log）。
- 原样收：OPUS INFO / SCINFO / SC / F2INFO / F2 行；cargo 的 test result 行（去掉 finished in 那一截）；用例 panic 的那一行（只留用例名）。
- CB 行（opus_sweep 每格一行）不原样收：按 (kind, k, root_hidden, M1 结局, 打中与否) 数格出 TALLY；打中（M1 做成且写行根落盘之前、
  或放开用户动作之后 R 的根还在环里时 R 的单元被写掉）的格各出一行 HIT；M1 被拒的格各出一行 REFUSED（带判官在置零镜像上的判定）；
  recF / roF / M1 落到的版本不全相同的格各出一行 DIVERGE；带层 0 汇总（L0 states=…）的格各出一行 L0。"""
import collections
import pathlib
import re
import sys

out = []
for path in map(pathlib.Path, sys.argv[1:]):
    name = path.name.removesuffix(".log")
    tally = collections.Counter()
    for line in path.read_text().splitlines():
        if line.startswith("test result:"):
            out.append(f"{name}\t{re.sub(r';? finished in [0-9.]+s', '', line)}")
            continue
        panic = re.match(r"thread '([^']+)' \(\d+\) panicked", line)
        if panic:
            out.append(f"{name}\tPANICKED {panic.group(1)}")
            continue
        if line.startswith(("OPUS INFO", "OPUS SC", "OPUS F2")):
            out.append(f"{name}\t{line}")
            continue
        if not line.startswith("OPUS CB "):
            continue
        f = dict(re.findall(r"(\w+)=(\S+)", line))
        m1 = f.get("M1", "?")
        cell = f"kind={f['kind']} k={f['k']} unit={f['unit']} rec_hidden={f['rec_hidden']} root_hidden={f['root_hidden']}"
        before = int(re.search(r"r_copies_overwritten=(\d+)", line).group(1)) if m1 == "Ok" else 0
        sweep = int(re.search(r"max_r_copies_overwritten_while_r_in_ring=(\d+)", line).group(1)) if m1 == "Ok" else 0
        hit = "hit" if before > 0 or sweep > 0 else "-"
        tally[(f["kind"], f["k"], f["root_hidden"], m1, hit)] += 1
        if hit == "hit":
            chk = re.search(r"chk=(\{[^}]*\})", line).group(1)
            out.append(f"{name}\tHIT {cell} effective={f.get('effective')} before_row_root={before} sweep_max={sweep} chk={chk}")
        if m1 != "Ok":
            out.append(f"{name}\tREFUSED {cell} M1={m1} judge_on_zeroed={f.get('judge_on_zeroed', '-')} judge_selected={f.get('judge_selected', '-')}")
        rec = re.search(r"recF=(\w+)\(([^ )]*)", line)
        ro = re.search(r"roF=(\w+)\(([^ )]*)", line)
        rec_v = rec.group(2) if rec and rec.group(1) != "Failed" else "Failed"
        ro_v = ro.group(2) if ro and ro.group(1) == "Ok" else "Err"
        m1_v = f.get("effective", m1)
        if len({rec_v, ro_v, m1_v}) > 1:
            out.append(f"{name}\tDIVERGE {cell} recF={rec_v} roF={ro_v} M1={m1_v}")
        l0 = re.search(r"(L0 states=.*)$", line)
        if l0:
            out.append(f"{name}\tL0 {cell} {l0.group(1)}")
    for (kind, k, root_hidden, m1, hit), count in tally.items():
        out.append(f"{name}\tTALLY kind={kind} k={k} root_hidden={root_hidden} M1={m1} r_units={hit} cells={count}")
for line in sorted(out):
    print(line)
