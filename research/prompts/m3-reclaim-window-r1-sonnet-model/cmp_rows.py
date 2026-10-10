import sys,re
mine=sys.argv[1]; armname=sys.argv[2]; out='/home/fy5090/code/singlefs/research/results/e139-tightened-floor-2026-10-04.out'
exp={}
for l in open(out):
    if l.startswith('E7RESULT name=w_activation arm='+armname+' '):
        m=re.search(r'variant=(\S+) s=(\d+) lost_disk=(\d) failures=(\d) fakes=(\d+) unrecoverable=(\d+) retention_violations=\d+ checks=(\d+)',l)
        exp[(m[1],m[2],m[3],m[4])]=(int(m[5]),int(m[6]),int(m[7]))
got={}
for l in open(mine):
    m=re.match(r'variant=(\S+) s=(\d+) lost_disk=(\d) failures=(\d) fakes=(\d+) unrecoverable=(\d+) checks=(\d+)',l)
    if m: got[(m[1],m[2],m[3],m[4])]=(int(m[5]),int(m[6]),int(m[7]))
bad=0;n=0
for k,v in got.items():
    n+=1
    if exp.get(k)!=v:
        bad+=1; print('DIFF',k,'mine',v,'out',exp.get(k))
print('compared',n,'diff',bad)
