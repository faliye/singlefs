import re
p='/tmp/claude-1000/m3-root-ring-code-r3/tree/crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs'
L=open(p,encoding='utf-8').read().split('\n')
def grab(start):
    s=''
    i=start
    while True:
        s+=L[i]
        if L[i].rstrip().endswith('";'): break
        i+=1
    return s
full=grab(108); quick=grab(123)
def nums(s):
    s=s.split('= "',1)[1]
    s=s.replace('\\','')
    return {k:int(v) for k,v in re.findall(r'(\w+)=(\d+)',s)}
f=nums(full); q=nums(quick)
print('full by-publish keys',len(f),'sum',sum(f.values()))
print('quick by-publish keys',len(q),'sum',sum(q.values()))
# 带单元普通发布 n 写：上一次轮换段 8 + 2^n -1 + 记录段 3 + 根槽段 1 = 2^n+11
for n in (16,24,28):
    print(n,2**n+11)
print('txg9 expected', 2**24+11, 'got', f['instance2_txg9'])
