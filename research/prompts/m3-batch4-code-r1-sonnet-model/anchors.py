import sys,re
F='/tmp/claude-1000/m3-batch5-freeze/tree/'
rows=open(F+'crates/mutations.tsv',encoding='utf-8').read().split('\n')
sel=list(range(1776,1803))+list(range(1803,1813))+[1819,1820,1821,173,174,177,867,1350,1595]+list(range(1660,1668))
for i in sel:
    f=rows[i-1].split('\t')
    name,file,orig,repl=f[0],f[1],f[2],f[3]
    o=orig.replace('\\n','\n'); 
    try:
        src=open(F+file,encoding='utf-8').read()
    except Exception as e:
        print(i,'MISSING',file); continue
    print(i,src.count(o),len(f),name[:50])
