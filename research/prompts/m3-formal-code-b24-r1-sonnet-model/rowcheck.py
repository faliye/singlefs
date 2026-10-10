import sys
root='/tmp/claude-1000/m3-formal-code-b24-r1/tree/'
for n,line in enumerate(open(root+'crates/mutations.tsv',encoding='utf8'),1):
    if n<2436 or n>2442: continue
    f=line.rstrip('\n').split('\t')
    print(n,len(f))
    name,path,orig,mut,cmd,test=f[:6]
    o=orig.replace('\\n','\n'); 
    src=open(root+path,encoding='utf8').read()
    print('  orig count',src.count(o),'mut count',src.count(mut.replace('\\n','\n')),'test:',test)
