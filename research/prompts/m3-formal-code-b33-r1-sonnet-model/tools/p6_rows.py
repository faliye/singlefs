#!/usr/bin/env python3
"""P6 静态核：这一轮新加 / 替换 / 删除的变异行（改动全文 _m3-formal-code-b33-r1-diff.md 里 mutations-append / mutations-delete 两节，
加 E156 第 5、6 次的 M35、M44–M47）在冻结副本 crates/mutations.tsv 里的情况：
  ① 恰好出现一次（整行逐字）；② 锚点在文件里恰好命中一次；③ cargo 参数里的 -p / --test / --bin / --lib 指到的目标文件在；
  ④ 必须红的测试名是那几个目标源码里的一个 fn、并且参数里 -- 之后的过滤串至少有一个是它名字的子串（点名的测试跑得到）；
  ⑤ 与表里别的行 原文 + 替换文 逐字相同的行（同一处同一改法）、以及这些行点名的测试；⑥ 被删的旧名在表里还在不在。
用法：p6_rows.py 冻结副本根 diff文件"""
import collections, os, re, sys
tree, diff_path = sys.argv[1], sys.argv[2]
table = [line.rstrip("\n") for line in open(os.path.join(tree, "crates/mutations.tsv"), encoding="utf-8")]
rows = [l.split("\t") for l in table]
unescape = lambda t: t.replace("\\n", "\n")
by_line = collections.defaultdict(list)
for n, l in enumerate(table, 1): by_line[l].append(n)
by_mutation = collections.defaultdict(list)
for n, r in enumerate(rows, 1):
    if len(r) == 6: by_mutation[(r[1], r[2], r[3])].append(n)
by_name = collections.defaultdict(list)
for n, r in enumerate(rows, 1): by_name[r[0]].append(n)

# 改动全文里的追加行与删除行
append, delete = [], []
section, grp, inblock = None, None, False
for l in open(diff_path, encoding="utf-8").read().split("\n"):
    if l.startswith("## "): grp = l[3:]
    elif l.startswith("### "): section = l[4:]
    elif l.startswith("````"): inblock = not inblock
    elif inblock and l.strip() and section == "mutations-append.tsv": append.append((grp, l))
    elif inblock and l.strip() and section == "mutations-delete.txt": delete.append((grp, l))
extra_names = [r[0] for r in rows if re.match(r"E156 M(35|44|45|46|47)（", r[0])]

def source_files(args):
    """-p 包 + --test/--bin/--lib → 源码文件列表"""
    toks = args.split()
    pkg = toks[toks.index("-p") + 1] if "-p" in toks else None
    head = toks[: toks.index("--")] if "--" in toks else toks
    files, problems = [], []
    base = os.path.join(tree, "crates", pkg) if pkg else None
    for i, t in enumerate(head):
        if t == "--test": files.append(os.path.join(base, "tests", head[i + 1] + ".rs"))
        if t == "--bin": files.append(os.path.join(base, "src", "bin", head[i + 1] + ".rs"))
        if t == "--lib":
            for root, _, names in os.walk(os.path.join(base, "src")):
                files += [os.path.join(root, x) for x in names if x.endswith(".rs") and "bin" not in root.split(os.sep)]
    return pkg, files

def check(grp, line, number_known=None):
    f = line.split("\t")
    hits = by_line.get(line, [])
    n = hits[0] if hits else by_name.get(f[0], [None])[0]
    r = rows[n - 1]
    src = os.path.join(tree, r[1])
    text = open(src, encoding="utf-8").read()
    anchor = text.count(unescape(r[2]))
    pkg, files = source_files(r[4])
    missing = [x for x in files if not os.path.exists(x)]
    body = "".join(open(x, encoding="utf-8").read() for x in files if os.path.exists(x))
    fn_found = bool(re.search(r"\bfn " + re.escape(r[5]) + r"\b", body))
    toks = r[4].split()
    filters = [t for t in toks[toks.index("--") + 1:] if not t.startswith("--")] if "--" in toks else []
    selected = (not filters) or any(t in r[5] for t in filters)
    same = [m for m in by_mutation[(r[1], r[2], r[3])] if m != n]
    named = {m: rows[m - 1][5] for m in same}
    print(f"{grp:15s} 行{n:5d} 整行出现{len(hits)}次 锚点命中{anchor} 目标文件缺{len(missing)} fn在{fn_found} 过滤串选中{selected} | 同一改法的别的行 {named if named else '无'} | {r[0][:46]}")

print("== 追加行（改动全文 mutations-append.tsv）", len(append))
for grp, l in append: check(grp, l)
print("== E156 第 5、6 次的变异行", len(extra_names))
for name in extra_names:
    n = by_name[name][0]; check("E156", table[n - 1])
print("== 删除行 mutations-delete.txt")
for grp, name in delete:
    print(grp, "旧名在表里的行：", by_name.get(name, []), "|", name)
dups = {k: v for k, v in by_name.items() if len(v) > 1}
print("== 表里变异名重复的行数", len(dups), list(dups)[:3])
