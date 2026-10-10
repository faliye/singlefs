import sys, re, json
diff = open('/home/fy5090/code/singlefs/research/prompts/_m3-batch10z-f-code-r1-diff.md', encoding='utf-8').read().split('\n')
def section(start_marker, end_marker):
    out=[]; on=False; fence=0
    i=0
    while i < len(diff):
        if diff[i].startswith(start_marker): on=True; i+=1; continue
        if on:
            if diff[i].startswith('````tsv') or diff[i].startswith('````'):
                fence+=1
                if fence==2: break
                i+=1; continue
            if fence==1 and diff[i].strip(): out.append(diff[i])
        i+=1
    return out
app = section('## 变异表追加', '')
rep = section('## 整行替换', '')
rows=[]
for kind, lst in (('append',app),('replace',rep)):
    for l in lst:
        f=l.split('\t')
        rows.append(dict(kind=kind,name=f[0],file=f[1],anchor=f[2].replace('\\n','\n'),repl=f[3].replace('\\n','\n'),cmd=f[4],test=f[5] if len(f)>5 else '', nfields=len(f)))
json.dump(rows, open(sys.argv[1],'w'), ensure_ascii=False, indent=1)
print(len(app), len(rep))
