import json,sys
o=json.load(open(sys.argv[1])); n=json.load(open(sys.argv[2])); show=sys.argv[3] if len(sys.argv)>3 else ""
flag=0
for a,b in zip(o,n):
    exp=b.get("expect"); fl=[]
    if exp is not None and b["rc"]!=exp: fl.append("改后≠期望")
    if exp is not None and a["rc"]!=exp: fl.append("改前≠期望")
    if a["rc"]!=b["rc"]: fl.append("改前≠改后")
    if fl: flag+=1
    if fl or show=="all":
        print(f"{'*' if fl else ' '} [{b['tag']}] {b['hook']}/{b['agent']} 期望={exp} 改前={a['rc']} 改后={b['rc']} {' '.join(fl)}\n      cmd={b['cmd']!r}"+(f"\n      改后原因={b['reason']}" if b['rc']==2 and show=='all' else "")+(f"\n      改后提示={b['note']}" if b['note'] else ""))
print("total",len(n),"flagged",flag)
