"""show.py <族> <树> [排除正则] [最多条数] : 列出该族在该树上判错（对裁判）的条，按 tag 分组计数，每组列前几条。语法错不算。"""
import sys, json, re, collections
fam, tree = sys.argv[1:3]; excl = re.compile(sys.argv[3]) if len(sys.argv) > 3 and sys.argv[3] else None; lim = int(sys.argv[4]) if len(sys.argv) > 4 else 4
cases = json.load(open(f"cases/cases-{fam}.json")); orc = json.load(open(f"res/{fam}.oracle.json")); r = json.load(open(f"res/{fam}.{tree}.json"))
groups = collections.defaultdict(list); tot = collections.Counter()
for c, o, x in zip(cases, orc, r):
    if o.get("noexec"): exp = c["expect"]
    else:
        if "syntax error" in o["err1"] or "unexpected" in o["err1"]: continue
        exp = 2 if (o["heavy"] if c["hook"] == "guard" else o["detached"]) else 0
    tot[c["tag"]] += 1
    if x["rc"] != exp:
        if excl and excl.search(c["cmd"]): continue
        groups[(c["tag"], "误拒" if x["rc"] == 2 else "漏拒")].append((c["cmd"], c["hook"], o.get("err1", "")))
for (tag, kind), v in sorted(groups.items()):
    print(f"== {tag} {kind} {len(v)}/{tot[tag]}")
    for cmd, hook, e in v[:lim]: print(f"   {hook}| {cmd!r}" + (f"  [bash:{e[:60]}]" if e else ""))
