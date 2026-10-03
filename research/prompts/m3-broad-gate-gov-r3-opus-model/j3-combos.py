#!/usr/bin/env python3
"""J3 第二条：第二轮 H1-7（结局组合下 --summary 两行与 gate-verdict 互核）在今天的驱动与门禁上再抽一次，这一次结局里加上升档判的抓到
（结果行 caught、「抓到它的测试」一段是 caught-by-timeout / caught-by-memory）。升档行的证据照 escalate_step 的写法造（两份日志末尾带驱动那一行、
带包装那一句，登记行的基线取这一版 baseline.tsv / baseline-peaks 里的），按概率不造（另一台机器导入的那种），看 gate-verdict 的判定与
「每条有结论、没有没红、状态是升档抓到的条都有这一版的登记行」是否一致。
用法：j3-combos.py <驱动> <gate_verdict.py（门禁 gate-verdict 那段内嵌 python 原样）> <草稿目录> <趟数> <种子>"""
import contextlib, hashlib, importlib.util, io, json, os, random, shutil, subprocess, sys
driver_path, verdict_path, work, trials, seed = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), int(sys.argv[5])
spec = importlib.util.spec_from_file_location("bm", driver_path); bm = importlib.util.module_from_spec(spec); spec.loader.exec_module(bm)
rng = random.Random(seed)
OUT = ["caught", "missed", "unviable", "timeout", "memory-cap", "stage-one-only", "driver-error", "esc-timeout", "esc-memory"]
fp = "ab" * 32
tallies, mismatch = {}, []
for trial in range(trials):
    home = os.path.abspath(os.path.join(work, "home")); shutil.rmtree(home, ignore_errors=True); os.makedirs(os.path.join(home, "tree"))
    vdir = os.path.join(home, "versions", fp); os.makedirs(os.path.join(vdir, "escalation"))
    open(os.path.join(vdir, "baseline.tsv"), "w").write("# step\toutcome\tseconds\tfinished_at\none-build\tok\t10.0\tt\n")
    open(os.path.join(vdir, "baseline-peaks.8.tsv"), "w").write(f"{1 << 30}\tt\t24G\tbroad-mutation baseline one-test\t8\n")
    k = rng.randint(1, 6)
    items = [{"sequence": s, "package": "p", "name": f"m{s}", "file": "f.rs"} for s in range(1, k + 1)]
    json.dump(items, open(os.path.join(vdir, "list.json"), "w"))
    json.dump({"fingerprint": fp, "commit": "c" * 40}, open(os.path.join(home, "tree.source"), "w"))
    rows, registry, evidence = [], [], {}
    for s in range(1, k + 1):
        if rng.random() < 0.15:
            continue
        seq = [("caught", bm.CARRIED_STAGE)] if rng.random() < 0.3 else []
        for _ in range(rng.randint(0 if seq else 1, 3)):
            seq.append((rng.choice(OUT), rng.choice(["one", "two"])))
        for number, (outcome, stage) in enumerate(seq):
            test = ""
            if outcome.startswith("esc-"):
                label = bm.ESCALATED_TIMEOUT_LABEL if outcome == "esc-timeout" else bm.ESCALATED_MEMORY_LABEL
                outcome, test = "caught", label
                if rng.random() < 0.8:      # 驱动写了证据；另 0.2 没有（导入的）
                    step, base, first, up, code = (("one-build", "10.0", "900", "3600", 253) if label == bm.ESCALATED_TIMEOUT_LABEL
                                                   else ("one-test", str(1 << 30), "24G", "36864M", 250))
                    logs = []
                    for attempt, limit in (("first", first), ("escalated", up)):
                        relative = f"escalation/{s}.{step}.{number}.{attempt}.log"
                        text = (f"run-with-memory-cap: {bm.wrapper_hit_text(code, limit)}\n"
                                f"{bm.ESCALATION_EVIDENCE_PREFIX} version={fp} sequence={s} step={step} attempt={attempt} limit={limit} exit={code}\n")
                        open(os.path.join(vdir, relative), "w").write(text)
                        logs += [relative, hashlib.sha256(text.encode()).hexdigest()]
                    registry.append([fp, str(s), f"m{s}", label, step, base, "8", first, up, "20", *logs, "t"])
                    evidence.setdefault(str(s), set()).add(label)
            rows.append([str(s), "p", f"m{s}", outcome, stage, test, "0.0", "0.0", "t", fp, ""])
    with open(os.path.join(vdir, "results.tsv"), "w") as h:
        h.write("\t".join(bm.RESULT_COLUMNS) + "\n" + "".join("\t".join(r) + "\n" for r in rows))
    with open(os.path.join(home, bm.ESCALATION_REGISTRY), "w") as h:
        h.write("# " + "\t".join(bm.ESCALATION_COLUMNS) + "\n" + "".join("\t".join(r) + "\n" for r in registry))
    lib = bm.Library(home); lib.load_source(for_run=False); lib.load_items()
    buffer, errors = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(buffer), contextlib.redirect_stderr(errors):
        code = bm.summarise(lib)
    out_path = os.path.join(work, "summary.txt"); open(out_path, "w").write(buffer.getvalue() + errors.getvalue())
    states = lib.item_states()
    last_decisive = {}
    for row in rows:
        if row[3] in bm.DECISIVE_OUTCOMES:
            last_decisive[row[0]] = row
    truth_green = True
    for s in range(1, k + 1):
        state = (states.get(str(s)) or bm.ItemState("none", False)).state
        if state not in ("caught", "unviable"):
            truth_green = False
        elif state == "caught" and last_decisive[str(s)][5] in bm.ESCALATED_LABELS and last_decisive[str(s)][5] not in evidence.get(str(s), set()):
            truth_green = False
    gate = subprocess.run([sys.executable, verdict_path, out_path, str(code), home, "", os.path.join(home, "tree.source"), driver_path], capture_output=True, text=True)
    inconsistent = "判定行、汇总行、计数、明细与退出码对不上" in gate.stdout
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
