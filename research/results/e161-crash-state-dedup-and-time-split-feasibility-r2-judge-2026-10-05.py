#!/usr/bin/env python3
# E161 第 2 次重跑登记（research/prompts/e161-r2-prereg.md）第六节的判法，只读产物。
# 用法：
#   judge.py compare <第 1 次产物> <第 2 次产物>      行 3：按登记表分类比方向（Q3a/Q3b/Q3c）
#   judge.py pcr3 <第 1 次产物> <草稿目录>             PCr3：自己对自己比、改两处再比
#   judge.py q1 <第 2 次产物>                          行 1：Q1b、Q1c（四个数逐段比）
#   judge.py q2c <第 2 次产物>                         行 2：Q2c 九样有没有对象
import sys, os
SKIP_NAMES = {"config", "done", "preflight", "anchor_segment", "stop_clause_tally"}
KEY_FIELDS = ["scope", "cell", "stream", "arm", "busy_zone", "side"]
ZERO = {"inconsistent", "recorded_consult_report_differs", "recorded_ignore_report_differs",
        "recorded_verdicts_differ", "walk_unit_reads_outside_normal_unit_reads", "content_collisions",
        "persisted_mismatches", "second_run_recovery_outcome_changed", "second_run_verdict_changed",
        "pc2a_recovery_keys_changed", "pc4a_checker_keys_changed", "pc2d_recovery_reports_changed",
        "pc4d_verdicts_changed", "inconsistent_without_removal"}
POSITIVE = {"constant_key_inconsistent", "hits", "discriminating_positions"}
RENAME = {"second_segment_14_pc3": "second_segment_16_pc3",
          "first_nine_segments_identical": "shared_prefix_segments_identical"}

def rows_of(path):
    out = []
    for ln, line in enumerate(open(path, encoding="utf-8"), 1):
        if line.startswith("E7RESULT "):
            kv = {}
            for t in line.rstrip("\n")[len("E7RESULT "):].split(" "):
                if "=" in t:
                    k, v = t.split("=", 1)
                    kv.setdefault(k, v)
            out.append((ln, kv))
    return out

def parse(path):
    rows, dup = {}, []
    for ln, kv in rows_of(path):
        name = kv.get("name")
        if name in SKIP_NAMES:
            continue
        key = (name,) + tuple(f"{f}={kv[f]}" for f in KEY_FIELDS if f in kv)
        if key in rows:
            dup.append((path, ln, key))
        rows[key] = (ln, kv)
    return rows, dup

def to_int(v):
    try:
        return int(v)
    except ValueError:
        return None

def klass(field, v, kv):
    if v in ("true", "false"):
        return ("布尔", v)
    n = to_int(v)
    if field in ZERO and n is not None:
        return ("钉零", "=0" if n == 0 else ">=1")
    if field in POSITIVE and n is not None:
        return ("要正", ">=1" if n >= 1 else "=0")
    if field == "distinct_outcomes" and n is not None:
        return ("结局种数", ">=2" if n >= 2 else "<=1")
    if field == "states_without_unit_reads" and n is not None:
        s = to_int(kv.get("states", ""))
        if s is not None:
            return ("读不到", "==states" if n == s else "<states")
    if kv.get("name") == "verdict" and len(field) == 2 and field[0] == "g" and field[1].isdigit():
        return ("G行判定", v)
    return None

def compare(pa, pb):
    ra, da = parse(pa)
    rb, db = parse(pb)
    for d in da + db:
        print("重复的键", d)
    flips, only_key, only_field, compared = [], [], [], {}
    for key in sorted(set(ra) | set(rb), key=lambda k: (k[0], k[1:])):
        if key not in ra or key not in rb:
            only_key.append((key, "只在第 1 次" if key in ra else "只在第 2 次"))
            continue
        la, ka = ra[key]
        lb, kb = rb[key]
        fa = {RENAME.get(f, f): (f, v) for f, v in ka.items()}
        fb = {RENAME.get(f, f): (f, v) for f, v in kb.items()}
        for f in sorted(set(fa) | set(fb)):
            if f not in fa or f not in fb:
                only_field.append((key, f, "只在第 1 次" if f in fa else "只在第 2 次"))
                continue
            ca, cb = klass(fa[f][0], fa[f][1], ka), klass(fb[f][0], fb[f][1], kb)
            if ca is None and cb is None:
                continue
            if ca is None or cb is None:
                flips.append((key, f, la, fa[f][1], lb, fb[f][1], "类型对不上"))
                continue
            compared[ca[0]] = compared.get(ca[0], 0) + 1
            if ca != cb:
                flips.append((key, f, la, fa[f][1], lb, fb[f][1], ca[0]))
    print("键数 第1次/第2次:", len(ra), len(rb))
    print("比了的字段数（按类）:", compared, "合计", sum(compared.values()))
    print("翻面格数:", len(flips))
    for k, f, la, va, lb, vb, c in flips:
        print(f"  翻面 [{c}] 键={' '.join(k)} 字段={f} 第1次(行{la})={va} 第2次(行{lb})={vb}")
    print("只在一边的键数:", len(only_key))
    for k, w in only_key:
        print("  不可比的键", ' '.join(k), w)
    print("只在一边的字段数:", len(only_field))
    for k, f, w in only_field:
        print("  不可比的字段", ' '.join(k), f, w)
    return len(flips)

def pcr3(pa, scratch):
    print("== PCr3 ①：第 1 次产物对它自己")
    n1 = compare(pa, pa)
    mod = os.path.join(scratch, "pcr3-modified.out")
    done_v = done_r = False
    with open(mod, "x") as out:
        for i, line in enumerate(open(pa, encoding="utf-8"), 1):
            if (not done_v) and "name=verdict" in line and " clause_S2_holds=true" in line:
                line = line.replace(" clause_S2_holds=true", " clause_S2_holds=false", 1)
                done_v = True
                print(f"== 改了判决行 行{i}：clause_S2_holds true -> false")
            elif (not done_r) and "name=reuse_arm" in line and " inconsistent=0 " in line:
                line = line.replace(" inconsistent=0 ", " inconsistent=3 ", 1)
                done_r = True
                print(f"== 改了 reuse_arm 行 行{i}：inconsistent 0 -> 3")
            out.write(line)
    print("== PCr3 ②：第 1 次产物对改了两处的副本")
    n2 = compare(pa, mod)
    print(f"== PCr3 结果：① 翻面 {n1} 格（要求 0）；② 翻面 {n2} 格（要求 2）")
    print("== PCr3", "通过" if (n1 == 0 and n2 == 2) else "不通过")

def q1(pb):
    A2 = [8, 3, 1, 8, 3, 1, 8, 16777215, 3, 1, 8]
    A3 = [16777215, 3, 1, 8, 8, 262143, 3, 1, 8, 65535, 3, 1, 8, 65535, 3, 1]
    rows = {"first": {}, "second": {}}
    for _, kv in rows_of(pb):
        if kv.get("name") == "anchor_segment":
            rows[kv["stream"]][int(kv["segment"])] = kv
    for st, exp, segs in (("first", A2, range(11)), ("second", A3, range(11, 27))):
        bad = one = 0
        for i, s in enumerate(segs):
            kv = rows[st][s]
            nums = [int(kv["counted_by_the_plan"]), int(kv["counted_by_the_crates"]),
                    int(kv["counted_by_the_formula"]), exp[i]]
            if len(set(nums)) != 1:
                bad += 1
                print("不等", st, s, nums)
            if kv["in_domain_counted_one_by_one"] == "true":
                one += 1
        print(st, "段数", len(segs), "四个数不全相等的段数", bad, "逐个数 true 的段数", one)
    print("second 的 anchor_segment 行数", len(rows["second"]), "three_agree 非 true 的段",
          [s for s, kv in rows["second"].items() if kv["three_agree"] != "true"])

def q2c(pb):
    R = [kv for _, kv in rows_of(pb)]
    sel = lambda name, **kw: [r for r in R if r.get("name") == name and all(r.get(k) == v for k, v in kw.items())]
    no = []
    for cell, n in (("first_small", 45), ("first_quick", 46), ("first_segment_head", 4096), ("second_quick", 52), ("second_segment_head", 4096)):
        g = sel("g1", cell=cell)
        ok = len(g) == 1 and int(g[0]["states"]) == n
        print("1", cell, "states", [x["states"] for x in g], "期望", n, "有对象" if ok else "没有对象")
        ok or no.append(("1", cell))
    v = int(sel("positive_control_refeed")[0]["refed_state_unit_positions"]); print("2 refed_state_unit_positions", v); v > 0 or no.append("2")
    v = int(sel("reuse_arm", cell="first_small", arm="k2_consult")[0]["distinct_outcomes"]); print("3 first_small k2_consult distinct_outcomes", v); v >= 2 or no.append("3")
    vals = {r["cell"]: int(r["distinct_outcomes"]) for r in sel("reuse_arm", arm="k4_full")}; print("4 k4_full distinct_outcomes", vals); max(vals.values()) >= 2 or no.append("4")
    g = sel("positive_control_timing"); print("5 timing 行数", len(g), "states", sorted({r["states"] for r in g}), "pc3_segment_holds", sorted({r.get("pc3_segment_holds") for r in g}))
    (len(g) == 6 and {r["states"] for r in g} == {"1000"} and {r.get("pc3_segment_holds") for r in g} == {"true"}) or no.append("5")
    g = sel("stop_clause_crates"); print("6a crates compared_states", [(r["cell"], r["compared_states"]) for r in g]); (len(g) == 3 and all(int(r["compared_states"]) > 0 for r in g)) or no.append("6a")
    g = sel("stop_clause_crates_per_state"); print("6b per_state states", [(r["cell"], r["states"]) for r in g]); (len(g) == 2 and all(r["states"] == "4096" for r in g)) or no.append("6b")
    g = sel("stop_clause_determinism"); print("7 control_states", [(r["cell"], r["control_states"]) for r in g]); (len(g) == 2 and all(int(r["control_states"]) > 0 for r in g)) or no.append("7")
    g = sel("positive_control_remove_one_position"); print("8 positions", [(r["cell"], r["arm"], r["positions"]) for r in g]); (len(g) > 0 and all(int(r["positions"]) > 0 for r in g)) or no.append("8")
    g = sel("positive_control_device_memory"); print("9 device_memory 行", [r["cell"] for r in g]); len(g) == 2 or no.append("9")
    print("没有对象的格数", len(no), no)

if __name__ == "__main__":
    {"compare": lambda a: compare(a[0], a[1]), "pcr3": lambda a: pcr3(a[0], a[1]),
     "q1": lambda a: q1(a[0]), "q2c": lambda a: q2c(a[0])}[sys.argv[1]](sys.argv[2:])
