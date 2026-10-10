#!/usr/bin/env python3
"""自拟变异（只在自己的副本里施加）：<副本根> <json 文件>，json 是列表，每项 {name, source, find, occurrence(0 起), replace, args}。
施加第 occurrence 次出现的 find，跑 cargo test <args>，还原并核 sha256；输出：名字、退出码、红的测试名（FAILED 行）、第一条 panic 头。"""
import hashlib, json, os, re, subprocess, sys
root, spec = sys.argv[1], json.load(open(sys.argv[2], encoding="utf-8"))
runner = os.environ["RUNNER"]
for m in spec:
    path = f"{root}/{m['source']}"
    original = open(path, encoding="utf-8").read(); digest = hashlib.sha256(original.encode()).hexdigest()
    idx = [x.start() for x in re.finditer(re.escape(m["find"]), original)]
    assert len(idx) > m["occurrence"], (m["name"], len(idx))
    i = idx[m["occurrence"]]
    open(path, "w", encoding="utf-8").write(original[:i] + m["replace"] + original[i + len(m["find"]):])
    try:
        p = subprocess.run(f"{runner} {m['args']}", shell=True, cwd=root, capture_output=True, text=True, timeout=230)
        t = p.stdout + p.stderr; code = p.returncode
    finally:
        open(path, "w", encoding="utf-8").write(original)
    ok = hashlib.sha256(open(path, "rb").read()).hexdigest() == digest
    failed = re.findall(r"^test (\S+) \.\.\. FAILED", t, re.M)
    res = [l for l in t.split("\n") if l.startswith("test result")]
    comp = "error[E" in t or "could not compile" in t
    panic = next((" ".join(x.strip() for x in t.split("\n")[k + 1:k + 3])[:160] for k, l in enumerate(t.split("\n")) if "panicked at" in l), "")
    print(json.dumps(dict(name=m["name"], occurrences=len(idx), applied=m["occurrence"], exit=code, compile_error=comp, failed=failed, result=res, first_panic=panic, restore="ok" if ok else "MISMATCH"), ensure_ascii=False), flush=True)
