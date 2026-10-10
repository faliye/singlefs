#!/usr/bin/env python3
"""把 rows.json 里第 lo..hi-1 行的变异施加到副本（只在自己草稿目录的副本里写）、跑点名的 harness 档测试、还原，记红在哪。
用法 mutate_rows.py <副本根> <rows.json> <lo> <hi> <结果文件.jsonl>
环境变量 RUNNER 是跑 cargo test 的命令前缀（经 run-with-memory-cap 与 capped）。checker-tier 档的行跳过（不跑）。
每行记：序号、名字、退出码、test result 行、第一条 panic 信息头 220 字、分类（assertion / 越界或算术 panic / 编译失败 / 绿=存活 / 其他）。
还原后核 sha256 与施加前相同，不同写 RESTORE_MISMATCH。"""
import hashlib, json, os, re, subprocess, sys
root, rows_path, lo, hi, out = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4]), sys.argv[5]
runner = os.environ["RUNNER"]
rows = json.load(open(rows_path, encoding="utf-8"))
tsv = {}
for r in open(f"{root}/crates/mutations.tsv", encoding="utf-8").read().split("\n"):
    c = r.split("\t")
    if len(c) == 6: tsv[c[0]] = c
def classify(text, code):
    if code == 0: return "绿（变异存活）"
    if "error[E" in text or "could not compile" in text: return "编译失败"
    m = re.search(r"panicked at [^\n]*\n([^\n]*)", text)
    msg = m.group(1) if m else ""
    if re.search(r"index out of bounds|out of range for slice|range end index|range start index|overflow|byte index|called `Option::unwrap\(\)` on a `None`|called `Result::unwrap\(\)`", text.split("panicked at",1)[-1][:600]) and not re.search(r"assertion", text.split("panicked at",1)[-1][:200]):
        return "越界/算术/unwrap panic"
    if "assertion" in text.split("panicked at",1)[-1][:300] or msg: return "断言（或自带消息 panic）"
    return "其他"
for k in range(lo, min(hi, len(rows))):
    row = rows[k]
    if row["tier"] != "harness":
        print(json.dumps(dict(k=k, name=row["name"][:60], skipped="checker-tier 档，不跑"), ensure_ascii=False)); continue
    name, source, find, repl, args, test = tsv[row["name"]]
    path = f"{root}/{source}"
    original = open(path, encoding="utf-8").read()
    digest = hashlib.sha256(original.encode()).hexdigest()
    f, g = find.replace("\\n", "\n"), repl.replace("\\n", "\n")
    assert original.count(f) == 1, name
    open(path, "w", encoding="utf-8").write(original.replace(f, g))
    try:
        p = subprocess.run(f"{runner} {args}", shell=True, cwd=root, capture_output=True, text=True, timeout=230)
        text, code = p.stdout + p.stderr, p.returncode
    except subprocess.TimeoutExpired as e:
        text, code = ((e.stdout or b"").decode() if isinstance(e.stdout, bytes) else (e.stdout or "")) + "TIMEOUT", 124
    finally:
        open(path, "w", encoding="utf-8").write(original)
    ok = hashlib.sha256(open(path, "rb").read()).hexdigest() == digest
    res = [re.sub(r"; finished in .*", "", l) for l in text.split("\n") if l.startswith("test result")]
    lines = text.split("\n")
    panic = ""
    for i, l in enumerate(lines):
        if "panicked at" in l:
            panic = " ".join(x.strip() for x in lines[i + 1:i + 4])[:220]; break
    loc = next((l.strip() for l in lines if "panicked at" in l), "")
    named_failed = bool(re.search(r"test [^\n]*" + re.escape(test) + r" \.\.\. FAILED", text))
    rec = dict(k=k, name=name[:70], exit=code, result=res, panic_location=loc[:200], first_panic=panic, kind=classify(text, code), named_test_failed=named_failed, compiled_pkg=("Compiling singlefs" in text), restore="ok" if ok else "RESTORE_MISMATCH")
    open(out, "a", encoding="utf-8").write(json.dumps(rec, ensure_ascii=False) + "\n")
    print(json.dumps(rec, ensure_ascii=False))
