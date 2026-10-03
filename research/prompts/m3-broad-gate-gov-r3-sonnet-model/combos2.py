#!/usr/bin/env python3
"""H1 第 7 条：驱动 summarise 在各种结局组合下打的两行，喂抽出来的 gate-verdict 那段 python（门禁 :344–:443 原样），看互核会不会误红、判定与「每条有结论且没红」是否一致。
用法：combos.py <驱动> <gate_verdict.py> <草稿目录> <趟数> <种子>"""
import contextlib, importlib.util, io, json, os, random, subprocess, sys
driver_path, verdict_path, work, trials, seed = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), int(sys.argv[5])
spec = importlib.util.spec_from_file_location("bm", driver_path); bm = importlib.util.module_from_spec(spec); spec.loader.exec_module(bm)
rng = random.Random(seed)
OUT = ["caught", "missed", "unviable", "timeout", "memory-cap", "stage-one-only", "driver-error"]
home = os.path.abspath(os.path.join(work, "home")); os.makedirs(os.path.join(home, "tree"), exist_ok=True)
fp = "ab" * 32
tallies = {}
mismatch = []
for trial in range(trials):
    k = rng.randint(1, 6)
    vdir = os.path.join(home, "versions", fp); os.makedirs(vdir, exist_ok=True)
    items = [{"sequence": s, "package": "p", "name": f"m{s}", "file": "f.rs"} for s in range(1, k + 1)]
    json.dump(items, open(os.path.join(vdir, "list.json"), "w"))
    json.dump({"fingerprint": fp, "commit": "c" * 40}, open(os.path.join(home, "tree.source"), "w"))
    rows = []
    for s in range(1, k + 1):
        kind = rng.random()
        if kind < 0.15:
            continue                                   # 没跑
        seq = []
        if rng.random() < 0.3:
            seq.append(("caught", bm.CARRIED_STAGE))   # 先有沿用行
        for _ in range(rng.randint(0 if seq else 1, 3)):
            seq.append((rng.choice(OUT), rng.choice(["one", "two"])))
        rng.shuffle(seq) if rng.random() < 0.2 else None
        for outcome, stage in seq:
            rows.append([str(s), "p", f"m{s}", outcome, stage, ("caught-by-timeout" if (outcome=="caught" and stage!=bm.CARRIED_STAGE and rng.random()<0.3) else ""), "0.0", "0.0", "t", fp, ""])
    with open(os.path.join(vdir, "results.tsv"), "w") as h:
        h.write("\t".join(bm.RESULT_COLUMNS) + "\n" + "".join("\t".join(r) + "\n" for r in rows))
    lib = bm.Library(home); lib.load_source(for_run=False); lib.load_items()
    buffer, errors = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(buffer), contextlib.redirect_stderr(errors):
        code = bm.summarise(lib)
    out_path = os.path.join(work, "summary.txt")
    open(out_path, "w").write(buffer.getvalue() + errors.getvalue())
    states = lib.item_states()
    last = {}
    for r in rows:
        if r[3] in ("caught", "missed", "unviable"): last[int(r[0])] = r
    truth_green = all((last.get(x) is not None and last[x][3] in ("caught", "unviable")) for x in range(1, k + 1)) and not any(last[x][5] == "caught-by-timeout" for x in last)
    gate = subprocess.run([sys.executable, verdict_path, out_path, str(code), home, "", os.path.join(home, "tree.source"), driver_path], capture_output=True, text=True)
    inconsistent = "判法失灵" in gate.stdout
    key = (code, gate.returncode, truth_green, inconsistent)
    tallies[key] = tallies.get(key, 0) + 1
    if inconsistent or (gate.returncode == 0) != truth_green:
        mismatch.append((trial, code, gate.returncode, truth_green, gate.stdout.strip().splitlines()[:3]))
print(f"trials={trials} seed={seed}")
for key in sorted(tallies):
    print(f"summary_exit={key[0]} gate_exit={key[1]} truth_green={key[2]} gate_says_inconsistent={key[3]} count={tallies[key]}")
print(f"mismatches={len(mismatch)}")
for entry in mismatch[:5]:
    print(entry)
