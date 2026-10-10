# 攻方原型的汇总：OPUSX1 行（步骤写成 步[那一刻 C 那一槽在根环表里的类]=结局）按 臂、故障、用户动作、结局、checker 归类计数。
import sys,re,collections,glob
c=collections.Counter()
for pattern in sys.argv[1:]:
    for path in sorted(glob.glob(pattern)):
        for l in open(path):
            if 'OPUSX1' not in l: continue
            l=l[l.index('OPUSX1'):]
            f=dict(re.findall(r'(\w+)=(\S+)',l))
            st=' '.join(f'{a}[{k}]:'+(b.split(':')[0].split('(')[0]+('/'+b.split(':')[1][:40] if b.startswith('Err') else '')) for a,k,b in re.findall(r'(unmount|write|rollback)\[([\w-]+)\]=(\S+)',l))
            st=re.sub(r'Err@\d+','Err@n',st)
            c[(f['arm'],f['fault'],f['mode'],st,'checker='+f['checker'])]+=1
for k,v in sorted(c.items()): print(v,*k)
