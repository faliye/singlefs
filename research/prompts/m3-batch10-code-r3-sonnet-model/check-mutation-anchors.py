import sys
rows=open('mutations.tsv',encoding='utf-8').read().split('\n')
targets=[i for i,l in enumerate(rows,1) if l.startswith('第10h批') or l.startswith('第10e批 重建分配器读根环那一遍一条有效根都没读出时不重读、当场拒') or l.startswith('第10e批 重读那一遍还读不出有效根时不拒') or l.startswith('第10e批 重建分配器读根环那一遍一条有效根都没读出时照第 17 批') or l.startswith('删除批第 16 批 GA：每块盘上最新有效根取最小时不滤被抛弃的根')]
for i in targets:
    f=rows[i-1].split('\t')
    if len(f)!=6: print(i,'fields',len(f)); continue
    name,path,orig,repl,cmd,test=f
    orig=orig.replace('\\n','\n'); 
    src=open('/tmp/claude-1000/m3-batch10-code-r3/tree/'+path,encoding='utf-8').read()
    print(i,src.count(orig),name[:40])
