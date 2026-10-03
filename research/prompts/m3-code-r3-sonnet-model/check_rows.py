import sys,re
root="/tmp/claude-1000/m3-code-r3/tree/"
rows=[l.rstrip("\n") for l in open(root+"crates/mutations.tsv",encoding="utf-8")]
print("total lines",len(rows))
new=[r for r in rows if r.startswith("m3-code-r2 ")]
print("m3-code-r2 rows",len(new))
for r in new:
    f=r.split("\t")
    name,path,orig,repl,args,test=f[:6]
    src=open(root+path,encoding="utf-8").read()
    o=orig.replace("\\n","\n"); n=repl.replace("\\n","\n")
    cnt=src.count(o)
    # test name existence
    tfile=None
    if "--test " in args:
        tn=re.search(r"--test (\S+)",args).group(1)
        tfile=root+"crates/singlefs-harness/tests/"+tn+".rs"
    else:
        tfile=root+"crates/singlefs-core/src/block_device.rs"
    tsrc=open(tfile,encoding="utf-8").read()
    ex=len(re.findall(r"fn "+re.escape(test)+r"\b",tsrc))
    print(f"{cnt} anchor_hits; fn_hits={ex}; fields={len(f)}; {name[:70]}")
