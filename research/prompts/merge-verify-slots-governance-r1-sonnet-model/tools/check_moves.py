#!/usr/bin/env python3
"""只读：照 lib-lag-baseline.py 文件头「搬迁登记」，对 path-moves.tsv 相对 HEAD 新增的行逐行核：
写法、出处在、生效（基准有旧前缀文件、没有新前缀文件）、实据（旧前缀下每份文件在新前缀同相对路径上在且被跟踪）、旧前缀此刻不在磁盘上；
再核生效的行之间串不串。用法：check_moves.py <仓根>"""
import os, re, subprocess, sys
root = sys.argv[1]
def git(*a):
    return subprocess.run(["git","-C",root,*a],capture_output=True,text=True)
base_files = set(git("ls-tree","-r","--name-only","HEAD").stdout.splitlines())
index_files = set(git("ls-files").stdout.splitlines())
ignored_cache = {}
def tracked_or_unignored(p):
    if p in index_files: return True
    if not os.path.isfile(os.path.join(root,p)): return False
    r = git("check-ignore","-q","--",p)
    return r.returncode != 0
def under(prefix, files):
    return sorted(f for f in files if f == prefix or f.startswith(prefix + "/"))
old_table = git("show","HEAD:.claude/gate.d/path-moves.tsv").stdout.splitlines()
with open(os.path.join(root,".claude/gate.d/path-moves.tsv"),encoding="utf-8") as f:
    new_table = f.read().splitlines()
old_set = set(old_table)
rows = []
for i,l in enumerate(new_table,1):
    if not l.strip() or l.lstrip().startswith("#"): continue
    rows.append((i,l,l in old_set))
added = [(i,l) for i,l,o in rows if not o]
print(f"新表内容行 {len(rows)}，HEAD 里没有的（新增或改过） {len(added)}")
effective = []
problems = 0
for i,l in added:
    cols = l.split("\t")
    msg = []
    if len(cols) != 4: msg.append(f"列数 {len(cols)}")
    else:
        old,new,date,src = cols
        if not re.fullmatch(r"\d{4}-\d\d-\d\d",date): msg.append("日期写法")
        if not re.fullmatch(r"records/[^/]+\.md",src) or not os.path.isfile(os.path.join(root,src)): msg.append(f"出处不在 {src}")
        for p in (old,new):
            if p.startswith(("/","./")) or p.endswith("/") or re.search(r"\s",p) or "/./" in p or "/../" in p: msg.append(f"前缀写法 {p}")
        oldfiles = under(old, base_files); newbase = under(new, base_files)
        eff = bool(oldfiles) and not newbase
        if not oldfiles: msg.append("基准里旧前缀下没有文件（不生效）")
        if newbase: msg.append(f"基准里新前缀下已有文件 {len(newbase)} 份，如 {newbase[0]}（不生效）")
        if eff:
            effective.append((i,old,new))
            miss = []
            for f in oldfiles:
                rel = new + f[len(old):]
                if not tracked_or_unignored(rel): miss.append(rel)
            if miss: msg.append(f"实据缺 {len(miss)} 份，如 {miss[0]}")
            if os.path.exists(os.path.join(root,old)): msg.append("旧前缀此刻还在磁盘上")
    if msg:
        problems += 1
        print(f"{i}: {'; '.join(msg)} :: {cols[0][:80]} -> {cols[1][:80] if len(cols)>1 else ''}")
print(f"逐行核：{len(added)} 行，有问题 {problems} 行；生效 {len(effective)} 行")
# 串：任一行的旧或新前缀等于、包含或落在另一行的旧或新前缀下面
def rel(a,b): return a==b or a.startswith(b+"/") or b.startswith(a+"/")
chain = []
for x in range(len(effective)):
    for y in range(len(effective)):
        if x==y: continue
        i1,o1,n1 = effective[x]; i2,o2,n2 = effective[y]
        for a,an in ((o1,"旧"),(n1,"新")):
            for b,bn in ((o2,"旧"),(n2,"新")):
                if rel(a,b) and (i1,i2,an,bn) not in chain:
                    chain.append((i1,i2,an,bn))
print(f"生效行之间串的 (行,行,前缀种类) {len(chain)} 处")
for c in chain[:20]: print("  串", c)
