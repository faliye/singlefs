import sys
root=sys.argv[1]
rows=[l.rstrip('\n') for l in open(root+'/crates/mutations.tsv',encoding='utf-8')]
for n in [3965]+list(range(3986,3995)):
    f=rows[n-1].split('\t')
    name,path,orig,rep=f[0],f[1],f[2],f[3]
    src=open(root+'/'+path,encoding='utf-8').read()
    o=orig.replace('\\n','\n')
    print(n,len(f),src.count(o),name[:50])
