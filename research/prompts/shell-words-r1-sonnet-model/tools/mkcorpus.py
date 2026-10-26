"""mkcorpus.py <仓根> <输出.json>: 从仓内 .claude/ records/ scripts/ 的 md 里抽围栏代码块与行内代码里的命令（去重），供 inproc.py 改前改后各判一遍。内容随仓走，不逐字可复现。"""
import re, glob, json, os, random
import sys
root=sys.argv[1]   # 仓根
files=glob.glob(root+"/.claude/**/*.md",recursive=True)+glob.glob(root+"/records/*.md")+glob.glob(root+"/*.md")+glob.glob(root+"/scripts/**/*.md",recursive=True)
files=[f for f in files if "/singlefs-ai-sop/" not in f or True]
cmds=set()
starts=("cargo","bash","sh ","python3","python ","cp ","mv ","ln ","install ","env ","time ","eval ","builtin ","function ","echo ","cat ","nice ","timeout ","SINGLEFS","source ",". ","cd ","git ","grep ","ls ","sleep ","until ","while ","for ","kill","pkill","flock","/usr/bin/time","systemd-run","tee ","sed ","awk ","find ","xargs ","export ","S=","x=","FOO=","capped","scripts/","./")
for f in files:
    try: text=open(f,encoding="utf-8").read()
    except: continue
    # fenced blocks
    for m in re.finditer(r"```[a-z]*\n(.*?)```",text,re.S):
        block=m.group(1).strip("\n")
        lines=block.split("\n")
        if len(lines)<=12 and len(block)<1500: cmds.add(block)
        for ln in lines:
            ln=ln.strip()
            if ln.startswith(starts) and len(ln)<600: cmds.add(ln)
    for m in re.finditer(r"`([^`\n]{6,300})`",text):
        s=m.group(1).strip()
        if s.startswith(starts): cmds.add(s)
cmds=sorted(cmds)
print(len(cmds))
json.dump(cmds,open(sys.argv[2],"w"),ensure_ascii=False)   # 输出路径
