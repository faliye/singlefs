"""滞后表按拆分族与 HEAD 比的独立重算：不 import 仓里任何代码，照 scripts/gate/lib_lag_split_family.py 文件头的三种拆法重写 family_owner 与之和比法，
读 <仓根>/.claude/gate.d/comment-history-lag.tsv 与 git show HEAD 里那一份，逐族打「入口 base=基准 cur_sum=此刻之和 members=几个路径 [RED]」，再列红的族与族外的路径。
用法：python3 -I family_sums.py <仓根>（只读）。基准取 HEAD 是推的：上游与 HEAD 同一个提交时 gate_diff_base gate 取 HEAD；给了 GATE_BASE 要自己换。"""
import subprocess, sys
root = sys.argv[1]
def rows(text):
    out = {}
    for line in text.splitlines():
        if not line or line.startswith("#"): continue
        c = line.split("\t")
        if len(c) >= 2 and c[1].strip().isdigit(): out[c[0]] = int(c[1])
    return out
base = rows(subprocess.run(["git","-C",root,"show","HEAD:.claude/gate.d/comment-history-lag.tsv"],capture_output=True,text=True).stdout)
cur = rows(open(root+"/.claude/gate.d/comment-history-lag.tsv").read())
def prefixes(owner):
    suf = owner[-3:]
    if owner.endswith(".rs"):
        name = owner.rpartition("/")[2]
        if name in ("main.rs","mod.rs"): return [owner[:len(owner)-len(name)]]
        return [owner[:-3]+"/"]
    directory,_,name = owner.rpartition("/")
    stem = name[:-3]; d = directory + "/" if directory else ""
    r = [(d, stem.replace("-","_")+"_", (".py",))]
    if suf == ".sh":
        r.append((d, stem+"-", (".py",".sh")))
        if directory == ".claude/gate.d": r.append((directory+"/lib/", stem+"-", (".py",".sh")))
    return r
def plen(owner, path):
    if owner == path: return len(path)
    if owner.endswith(".rs"):
        p = prefixes(owner)[0]
        return len(p) if path.endswith(".rs") and path.startswith(p) else None
    best=None
    if owner[-3:] not in (".py",".sh"): return None
    for d,pre,sufs in prefixes(owner):
        if not path.startswith(d) or not path.endswith(sufs): continue
        n = path[len(d):]
        if "/" in n: continue
        if n.startswith(pre) and n[len(pre):-3]:
            best = max(best or 0, len(d)+len(pre))
    return best
def owner_of(path):
    if path in base: return path
    c = {o:plen(o,path) for o in base}
    c = {o:l for o,l in c.items() if l is not None}
    return max(c, key=lambda o:(c[o],len(o),o)) if c else None
groups={}
for p,n in cur.items():
    groups.setdefault(owner_of(p),[]).append((p,n))
for o,rs in sorted(groups.items(), key=lambda x: str(x[0])):
    tot=sum(n for _,n in rs)
    allowed = base.get(o,0)
    flag = "RED" if tot>allowed else ""
    print(f"{o}\tbase={allowed}\tcur_sum={tot}\tmembers={len(rs)}\t{flag}")
print("---- members of None / red groups")
for o,rs in groups.items():
    tot=sum(n for _,n in rs)
    if o is None or tot>base.get(o,0):
        for p,n in rs: print(o,p,n)
print("---- base rows with no member now")
for b in base:
    if b not in groups: print("no-member", b)
