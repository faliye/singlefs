"""Q3 第 3 问：今天的丁（ding_today）在第一轮攻方模型那一套历史上的结局，推的模型。
历史：第一轮攻方模型 research/prompts/m3-close-b19-reason-r1-opus-model/ 的两种（照原样拷来的 history_ext.py 与按路径导入的
第二轮攻方模型 research/prompts/m3-close-b19-design-r2-opus-model/q7/q7_cost_model.py），参数与第一轮相同（PARAMS、DEVS）。
用法：python3 q3_run.py <仓根> <段数> q2|q2ext <写者> <S> <J>
每条抬 F 的根四个变体各判一次：today（上界 4096）、no_bound（不设上界，封顶 65536 种）、genesis_known（第 0 代根那一种的 state 取格式认得出的值）、
genesis_txg_device（第 0 代根的几份都按写者的口径落区域 0 的盘，镜像上坐着的几份也一样）。
"""
import importlib.util
import os
import sys
import time
from collections import Counter
import history_ext as hx
from ding_today import judge_raise

PARAMS = dict(p_nonempty=0.35, p_lost_root=0.04, p_clean_crash=0.03, p_raise=0.08)
DEVS = (0, 1, 0)
R = hx.R
VARIANTS = (("today", dict()), ("no_bound", dict(max_rings=None)), ("genesis_known", dict(genesis_state=0)),
            ("genesis_txg_device", dict(genesis_device="txg")))


def load_round2(repo):
    path = os.path.join(repo, "research/prompts/m3-close-b19-design-r2-opus-model/q7/q7_cost_model.py")
    spec = importlib.util.spec_from_file_location("round2_opus_q7", path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def image_of(h, S, genesis_device):
    slots = {}
    for (region, slot), t in h.ring.items():
        v = h.versions[t]
        dev = DEVS[region]
        if t == 0 and genesis_device == "txg":
            dev = DEVS[0]
        slots[(region, slot)] = (t, v["inst"], v["F"], v["state"], dev)
    jv = {}
    for t in h.journal:
        v = h.versions[t]
        jv.setdefault(t, [])
        if (v["inst"], v["F"], v["state"]) not in jv[t]:
            jv[t].append((v["inst"], v["F"], v["state"]))
    return slots, jv


def main():
    repo, seeds, kind, writer, S, J = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4], int(sys.argv[5]), int(sys.argv[6])
    m = load_round2(repo)
    tally = Counter()
    started = time.time()
    for seed in range(seeds):
        steps = m.rand_steps(seed)
        if kind == "q2ext":
            h = hx.History2(seed, S, J, DEVS, steps, writer=writer, gen0=True, p_fallback=0.02, **PARAMS)
        else:
            h = m.History(seed, S, J, DEVS, steps, writer=writer, **PARAMS)
        images = {gd: image_of(h, S, gd) for gd in ("slot", "txg")}
        for r in sorted({t for t in h.ring.values()}):
            if r not in h.raises:
                continue
            legal = "legal" if h.raises[r][3] else "illegal"
            per = {}
            for name, var in VARIANTS:
                slots, jv = images["txg" if var.get("genesis_device") == "txg" else "slot"]
                root = slots[next(k for k, x in slots.items() if x[0] == r)]
                j = judge_raise(slots, jv, h.rows, root, DEVS, R, S, **var)
                if j is None:
                    break
                per[name] = j
            if not per:
                continue
            tally[(legal, "raise_roots")] += 1
            if per["today"]["window"] == "holds_undetermined":
                tally[(legal, "reaches_ding")] += 1
            for name, _ in VARIANTS:
                v, why = per[name]["final"]
                tally[(legal, name, v)] += 1
                if v == "partial":
                    tally[(legal, name, "why", why)] += 1
            if per["today"]["final"] == ("partial", "combos"):
                tally[(legal, "combos_then_no_bound", per["no_bound"]["final"][0])] += 1
            if per["today"]["final"] != per["genesis_known"]["final"]:
                tally[(legal, "genesis_known_changes", per["today"]["final"][0] + ">" + per["genesis_known"]["final"][0])] += 1
            if per["today"]["final"] != per["genesis_txg_device"]["final"]:
                tally[(legal, "genesis_txg_device_changes", per["today"]["final"][0] + ">" + per["genesis_txg_device"]["final"][0])] += 1
                if (legal, "example_device") not in tally:
                    tally[(legal, "example_device")] = 1
                    print(f"name=q3_example_genesis_device kind={kind} writer={writer} S={S} J={J} seed={seed} r={r} legal={legal} "
                          f"today={per['today']['final']} writer_device={per['genesis_txg_device']['final']} ring={sorted(h.ring.items())} rows={h.rows}")
            if legal == "legal" and per["today"]["final"][0] == "red" and (legal, "example_false_red") not in tally:
                tally[(legal, "example_false_red")] = 1
                print(f"name=q3_example_false_red kind={kind} writer={writer} S={S} J={J} seed={seed} r={r} ring={sorted(h.ring.items())} rows={h.rows}")
            tally[(legal, "max_rings")] = max(tally[(legal, "max_rings")], per["no_bound"]["rings"])
    for legal in ("legal", "illegal"):
        n = tally[(legal, "raise_roots")]
        if n == 0:
            continue
        parts = [f"name=q3_today kind={kind} writer={writer} S={S} J={J} seeds={seeds} {legal}_raise_roots={n} reaches_ding={tally[(legal, 'reaches_ding')]} max_rings_no_bound={tally[(legal, 'max_rings')]}"]
        for name, _ in VARIANTS:
            fin = " ".join(f"{v}={tally[(legal, name, v)]}" for v in ("holds", "red", "partial", "not_judged"))
            why = ",".join(f"{w}:{tally[(legal, name, 'why', w)]}" for w in
                           ("combos", "too_many", "disagree", "between", "no_valid_root", "journal0", "journal2", "journal3")
                           if tally[(legal, name, "why", w)])
            parts.append(f"{name}[{fin} | partial_why {why or '-'}]")
        parts.append("combos_then_no_bound[" + " ".join(f"{v}={tally[(legal, 'combos_then_no_bound', v)]}" for v in ("holds", "red", "partial")) + "]")
        ch = [k for k in tally if k[0] == legal and k[1] in ("genesis_known_changes", "genesis_txg_device_changes")]
        parts.append("changes[" + (",".join(f"{k[1]}:{k[2]}={tally[k]}" for k in sorted(ch)) or "-") + "]")
        print(" ".join(parts))
    print(f"seconds={time.time() - started:.1f}", file=sys.stderr)


if __name__ == "__main__":
    main()
