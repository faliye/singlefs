import sys
row=int(sys.argv[1])
tsv=open('/tmp/claude-1000/m3-batch10-code-r3/tree/crates/mutations.tsv',encoding='utf-8').read().split('\n')[row-1].split('\t')
name,path,orig,repl,cmd,test=tsv
orig=orig.replace('\\n','\n'); repl=repl.replace('\\n','\n')
src='/tmp/claude-1000/m3-batch10-code-r3/tree/'+path
dst='/tmp/claude-1000/m3-batch10-code-r3-sonnet/work/'+path
s=open(src,encoding='utf-8').read()
assert s.count(orig)==1
open(dst,'w',encoding='utf-8').write(s.replace(orig,repl))
print(name[:60]); print(cmd); print(test)
