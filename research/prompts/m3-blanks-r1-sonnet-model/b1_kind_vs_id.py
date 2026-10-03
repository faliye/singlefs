#!/usr/bin/env python3
"""B1：树的种类码 9..12 与树 ID 11、12 同值，有没有读者靠「错开」分辨。
只读冻结副本的 crates/ 源码，不与实现共用代码。三段：
 ① 扫每个 Rust 函数体，找「同时提到种类码与树 ID」的函数，逐个列出；
 ② 码取值方案 × 树 ID 空间（第一次发布 11..22，之后水位只增）的同值表；
 ③ 三个候选各要同步的出现处数（grep 计数）。
"""
import re, sys, pathlib, itertools
ROOT = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "/tmp/claude-1000/m3-blanks-r1/tree/crates")

def fn_bodies(text):
    """粗切：从 `fn 名` 起按大括号配平取函数体（注释与字符串里的括号忽略不计，够用）。"""
    out = []
    for m in re.finditer(r'\bfn\s+([A-Za-z0-9_]+)', text):
        i = text.find('{', m.end())
        semi = text.find(';', m.end())
        if i < 0 or (0 <= semi < i):
            continue
        depth, j = 0, i
        while j < len(text):
            c = text[j]
            if c == '{': depth += 1
            elif c == '}':
                depth -= 1
                if depth == 0: break
            j += 1
        out.append((m.group(1), text[i:j + 1], text.count('\n', 0, m.start()) + 1))
    return out

KIND = re.compile(r'\bTREE_KIND_[A-Z_]+|\.kind\b|\bkind\b\s*[:=]')
IDENT = re.compile(r'\bTREE_IDENTIFIER_[A-Z_]+|TreeIdentifier\b|\btree_identifier\b|\bhead_identifier\b')
print("== ① 同一函数体里既提种类码又提树 ID 的函数（冻结副本 crates/ 下全部 .rs）")
mixed = []
for p in sorted(ROOT.rglob("*.rs")):
    t = p.read_text(encoding="utf-8")
    for name, body, line in fn_bodies(t):
        if KIND.search(body) and IDENT.search(body):
            mixed.append((str(p.relative_to(ROOT)), line, name))
print(f"mixed_functions={len(mixed)}")
# 再在这些函数里找「一个量既当种类又当树 ID 用」的写法：种类转 u64、树 ID 转 u16，或两者直接相等比较
cross = re.compile(r'kind\s+as\s+u64|u64::from\(\s*[a-z_.]*kind|tree[a-z_.]*\.0\s+as\s+u16|u16::try_from\([a-z_.]*tree|[a-z_.]*kind\s*==\s*[a-z_.]*tree|[a-z_.]*tree[a-z_.]*\s*==\s*[a-z_.]*kind')
hits = []
for p in sorted(ROOT.rglob("*.rs")):
    t = p.read_text(encoding="utf-8")
    for n, line in enumerate(t.splitlines(), 1):
        if cross.search(line):
            hits.append((str(p.relative_to(ROOT)), n, line.strip()[:120]))
print(f"cross_use_lines={len(hits)}")
for h in hits: print("  ", *h)
for f, line, name in mixed[:40]: print(f"  mixed {f}:{line} fn {name}")
print("== ② 取值方案 × 树 ID：同值的格数")
ids_first = list(range(11, 23))          # 第一次发布后发出的 12 个树 ID（水位 23）
plans = {"甲 9..12": [9, 10, 11, 12], "乙-1 高段 0x8001..0x8004": [0x8001, 0x8002, 0x8003, 0x8004],
         "乙-2 16..19": [16, 17, 18, 19], "丙 码 9..12 + ID 从 65536 起": [9, 10, 11, 12]}
for name, codes in plans.items():
    base = 65536 if name.startswith("丙") else 11
    ids = [base + k for k in range(12)]
    same_first = sorted(set(codes) & set(ids))
    # 水位继续涨：快照、克隆各占一个树 ID，涨到 2^16 之内的哪些码迟早被某个树 ID 取到
    reachable = [c for c in codes if c >= base]
    print(f"{name}: 第一次发布后同值={same_first} 水位涨到 65535 前迟早撞上的码={reachable}")
print("种类 1..8 对树 ID：最小树 ID 11 > 8，用一条大小比较就能分开（范围分离）；9、10 是空档，只剩 2 个，装不下 4 棵新树")
print("== ③ 三个候选各要同步的出现处（grep 计数）")
def count(pat, glob="*.rs"):
    n = 0; files = set()
    for p in ROOT.rglob(glob):
        c = len(re.findall(pat, p.read_text(encoding="utf-8")))
        if c: n += c; files.add(p)
    return n, len(files)
for pat, label in [(r'TREE_KIND_(DELETED_INODES|TOMBSTONE|LINEAGE|LOGGED_OPERATIONS)', "四个新种类常量（候选甲、乙都要改取值的地方；按名引用，改常量取值不动这些行）"),
                   (r'TREE_IDENTIFIER_[A-Z_]+', "树 ID 常量按名引用（丙改取值也不动这些行）"),
                   (r'\b(11|12|13|14|15|16|17|18)\b', "裸数 11..18（含无关处，上界）")]:
    print(f"{label}: 次数,文件数={count(pat)}")
