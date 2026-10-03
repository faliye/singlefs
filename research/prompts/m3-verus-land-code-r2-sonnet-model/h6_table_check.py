#!/usr/bin/env python3
"""H6：钉住的表逐行对 Rust 装置 --pinned-cases 的读法（解析 + 重算 ①）与表头约定。只读，不跑装置。
用法：h6_table_check.py <表> <装置源码>"""
import hashlib, re, sys, collections
table, source = sys.argv[1], sys.argv[2]
src = open(source, encoding="utf-8").read()
labels = dict(re.findall(r'^\s+(\w+) => "([^"]+)",$', src[src.index("enum Domain {"):src.index("enum Family")], re.M))
smallest = {}
body = src[src.index("const fn smallest_size(self)"):src.index("const fn specification_self_check")]
for arm in re.findall(r"((?:Self::\w+\s*\|?\s*)+)=> (\d+),", body):
    for v in re.findall(r"Self::(\w+)", arm[0]):
        smallest[labels[v]] = int(arm[1])
def py_state(line):
    m = re.search(r"(?:^| )state=(\S+)$", line)
    return m.group(1).split(";first=")[0] if m else None
def rust_state(line):
    # 装置 input_fingerprint_of：rsplit_once(" state=")，空或含空白交 None，再截 ;first=
    if " state=" not in line: return None
    t = line.rsplit(" state=", 1)[1]
    if t == "" or any(c.isspace() for c in t): return None
    return t.split(";first=", 1)[0]
fp = lambda d, n, s: hashlib.sha256(f"{d}|{n}|{s}".encode()).hexdigest()
rows = []
for number, raw in enumerate(open(table, encoding="utf-8").read().split("\n"), 1):
    if raw.strip() and not raw.startswith("#"):
        rows.append((number, raw.split("\t")))
bad = collections.Counter(); examples = []
dom_n = collections.Counter()
for number, f in rows:
    if len(f) != 8: bad["列数不是八"] += 1; continue
    d, n, o = f[1], f[2], f[3]
    dom_n[(d, n)] += 1
    if d not in smallest: bad["②不在装置的域名表"] += 1; examples.append((number, "域", d)); continue
    if not n.isdigit() or not (smallest[d] <= int(n) <= 12): bad["③越界"] += 1; examples.append((number, "N", n))
    if not o.isdigit(): bad["④不是非负整数"] += 1
    if not f[6].startswith("E7RESULT name=state "): bad["⑦不以 E7RESULT name=state 起头"] += 1
    if f" domain={d} " not in f[6]: bad["⑦里的 domain= 与 ② 不同"] += 1
    if f" n={n} " not in f[6]: bad["⑦里的 n= 与 ③ 不同"] += 1
    if f" ordinal={o} " not in f[6]: bad["⑦里的 ordinal= 与 ④ 不同"] += 1
    if not f[7].startswith("E7RESULT name=state "): bad["⑧不以 E7RESULT name=state 起头"] += 1
    ps, rs = py_state(f[6]), rust_state(f[6])
    if ps != rs: bad["python 与装置的 state= 取法不同"] += 1; examples.append((number, "state 取法", (ps, rs)))
    if rs is None or fp(d, n, rs) != f[0]: bad["①重算不等"] += 1
    if f[7] != f[6] and False: pass
print(f"数据行 {len(rows)}；域名表 {len(smallest)} 个域；不同 (域,N) {len(dom_n)} 个；最大 N={max(int(n) for (_, n) in dom_n)}")
print("问题：", dict(bad) if bad else "无")
print("例：", examples[:5])
print("每个 (域,N) 钉了几行（取最多 5 个）：", dom_n.most_common(5))
