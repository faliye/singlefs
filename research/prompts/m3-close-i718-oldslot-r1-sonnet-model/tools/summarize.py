#!/usr/bin/env python3
"""把 probe_i718_oldslot 各变体各分组的日志（<日志目录>/<分组>-<变体>.log 等）汇成几张表。只读日志，不判对错。
用法：summarize.py <日志目录> [变体…]"""
import re
import sys
from collections import OrderedDict

ROW = re.compile(r"ROW group=(\S+) label=(.*?) newest_txg=(\d+) F=(\d+) t=(\d+) p=(\d+) ring_min=(\d+) ring_len=(\d+) I-7\.18=(\S+) I-7\.19=(\S+) other_red=(\[.*?\])")
VARIANTS = ["baseline", "a", "a_seed", "b", "b_strict"]
SHORT = {"成立": "Y", "红": "R", "不适用": "N", "部分没判": "P"}


def read(directory, name):
    try:
        return open(f"{directory}/{name}.log", encoding="utf-8").read()
    except FileNotFoundError:
        return None


def rows(directory, group, variant):
    text = read(directory, f"{group}-{variant}")
    if text is None:
        return None
    return [m.groups() for m in (ROW.search(line) for line in text.splitlines()) if m]


def main():
    directory = sys.argv[1]
    variants = sys.argv[2:] or VARIANTS
    print("== 一 逐 N 的目标态与对照态（I-7.18 判定；末列按变体顺序：" + " / ".join(variants) + "；成立=Y 红=R 不适用=N）")
    per_variant = {v: rows(directory, "group_next_slot_states", v) for v in variants}
    if all(per_variant[v] for v in variants):
        for index in range(len(per_variant[variants[0]])):
            first = per_variant[variants[0]][index]
            cells = "".join(SHORT[per_variant[v][index][8]] for v in variants)
            print(f"  {first[1]:<44} t={first[4]:>3} p={first[5]:>3} ring_min={first[6]:>3} ring_len={first[7]:>2}  {cells}")
    print("== 二 纯掉电：满环上 N+1 那次发布的每个前缀（崩点）盘面，各变体下 I-7.18 不成立的前缀数")
    for v in variants:
        prefix = rows(directory, "group_every_prefix_of_the_publish_on_a_full_ring", v)
        if prefix is None:
            continue
        by_n = OrderedDict()
        for r in prefix:
            n = r[1].split()[0]
            by_n.setdefault(n, [0, 0])
            by_n[n][0] += 1
            if r[8] != "成立":
                by_n[n][1] += 1
        print(f"  {v:<9} " + "  ".join(f"{n}: {total} 个前缀里 {bad} 个不成立" for n, (total, bad) in by_n.items()))
    print("== 三 错 t 镜像：N=14 的满环，最新根的 t 扫 0..16，各 p 下 I-7.18 判成立的 t；末行 I-3.1 红/绿（只看 p=255，与变体无关）")
    for group in ["sweep_target", "sweep_intact", "sweep_second_oldest_zeroed"]:
        print(f"  盘面 {group}")
        for v in variants:
            sweep = rows(directory, "group_wrong_t_and_p_mirrors", v)
            if sweep is None:
                continue
            accepted = OrderedDict()
            i31_green = []
            for r in sweep:
                if r[0] != group:
                    continue
                m = re.match(r"t=(\d+) p=(\d+)", r[1])
                t, p = int(m.group(1)), int(m.group(2))
                d = accepted.setdefault(p, {"Y": [], "R": [], "N": []})
                d[SHORT[r[8]]].append(t)
                if p == 255 and "I-3.1" not in r[10]:
                    i31_green.append(t)
            cells = []
            for p, d in accepted.items():
                cells.append(f"p={p}: 成立{d['Y']}" + (f" 不适用(全部 {len(d['N'])} 个 t)" if d["N"] else ""))
            print(f"    {v:<9} " + "; ".join(cells))
            if v == variants[0]:
                print(f"    （I-3.1 在 p=255 下判绿的 t：{i31_green}）")
    print("== 四 年轻的环（没满）里最旧那条根的槽事后写成全零")
    for v in variants:
        young = rows(directory, "group_young_ring_oldest_root_lost_afterwards", v)
        if young is None:
            continue
        print(f"  {v}: " + "; ".join(f"{r[1].split()[0]} {r[8]}" for r in young if "对照" not in r[1]))
    print("== 五 带空发布的历史（偶数 txg 空发布）：目标态的 t 扫描 0..8、p=255；各变体下 I-7.18 判成立的 t，与 I-3.1 判绿的 t（与变体无关）")
    for v in variants:
        sweep = rows(directory, "group_wrong_t_mirrors_on_a_history_with_empty_publishes", v)
        if not sweep:
            continue
        for n in ["N=14", "N=15"]:
            rows_of_n = [r for r in sweep if r[0] == "empty_target_sweep" and r[1].startswith(n)]
            holds = [int(re.search(r"t=(\d+)", r[1]).group(1)) for r in rows_of_n if r[8] == "成立"]
            not_applicable = [r for r in rows_of_n if r[8] == "不适用"]
            greens = [int(re.search(r"t=(\d+)", r[1]).group(1)) for r in rows_of_n if "I-3.1" not in r[10]]
            honest = [r[4] for r in sweep if r[0] == "empty_target" and r[1].startswith(n)]
            print(f"  {v:<9} {n}: 真写者的 t={honest}；I-7.18 判成立的 t={holds}" + (f"、不适用 {len(not_applicable)} 个 t" if not_applicable else "") + f"；I-3.1 判绿的 t={greens}")
    print("== 六 掉一块盘的降级镜像（I-7.18 / I-3.1 / I-7.19）")
    for v in variants:
        text = read(directory, f"group_a_device_missing-{v}")
        if text:
            for line in text.splitlines():
                if "ROWDEGRADED" in line:
                    print(f"  {v:<9} {line[line.index('ROWDEGRADED'):]}")
    print("== 六之二 第二圈以后（N=26，N+1=27 的落点位置 3，那一槽住着 txg 15）目标态的错 t 镜像：各 p 下 I-7.18 判成立的 t；真写者的 t；I-3.1 在 p=255 下判绿的 t")
    for v in variants:
        sweep = rows(directory, "group_second_revolution_wrong_t_and_p_mirrors", v)
        if not sweep:
            continue
        accepted = OrderedDict()
        greens = []
        for r in sweep:
            if r[0] != "second_revolution_sweep":
                continue
            m = re.match(r"t=(\d+) p=(\d+)", r[1])
            tt, pp = int(m.group(1)), int(m.group(2))
            d = accepted.setdefault(pp, {"Y": [], "N": 0})
            if r[8] == "成立":
                d["Y"].append(tt)
            elif r[8] == "不适用":
                d["N"] += 1
            if pp == 255 and "I-3.1" not in r[10]:
                greens.append(tt)
        honest = [r[4] for r in sweep if r[0] == "second_revolution_honest"]
        print(f"  {v:<9} 真写者 t={honest}; " + "; ".join(f"p={pp}: 成立{d['Y']}" + (f" 不适用{d['N']}个t" if d["N"] else "") for pp, d in accepted.items()) + f"; I-3.1 判绿的 t(p=255)={greens}")
    print("== 七 三个场景交出的镜像（investigator 的探针）：每个变体下判红的镜像数 / 总数")
    for v in variants:
        cells = []
        for sc in ["probe_plain", "probe_torn", "probe_zeroed"]:
            text = read(directory, f"scen-{sc}-{v}")
            if text is None:
                continue
            lines = [l for l in text.splitlines() if "PROBE_IMAGE" in l]
            red = [l for l in lines if "violated=[]" not in l]
            names = ["+".join(sorted(set(re.findall(r'\("(I-[0-9.]+)"', l.split(" violated=", 1)[1].split(" named=", 1)[0])))) for l in red]
            cells.append(f"{sc}: {len(red)}/{len(lines)} {names}")
        print(f"  {v:<9} " + "; ".join(cells))
    print("== 八 既有回归（测试结果行）")
    for v in variants:
        cells = []
        for name in ["rootrecord", "knownbad", "checkerlib"]:
            text = read(directory, f"regress-{name}-{v}")
            if text:
                result = [l for l in text.splitlines() if l.startswith("test result")]
                cells.append(f"{name}: {result[-1][:60] if result else '没有结果行'}")
        print(f"  {v:<9} " + " | ".join(cells))


main()
