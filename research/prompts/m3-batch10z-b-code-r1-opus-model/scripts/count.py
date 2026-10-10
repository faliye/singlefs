# 攻方原型的计数：X1 按 臂 × 故障，F1 按 臂 × 是否第 1 形 × 故障变体，数历史条数、有拒的、C/R 仍是被抛弃根时拒的、checker 红的、R 的单元被写掉的。
import re,glob,sys,collections
D=sys.argv[1]
print('# X1')
for arm in ['today','slot']:
  for fault in ['slot','account','none']:
    n=ref=refab=red=0
    for p in glob.glob(f'{D}/out/x1-{arm}-s*-{fault}.log'):
      for l in open(p):
        if 'OPUSX1' not in l: continue
        n+=1
        if 'checker=-' not in l: red+=1
        errs=re.findall(r'(unmount|write)\[(\w+)\]=Err',l)
        ref+=bool(errs); refab+=any(k=='AbandonedRoot' for _,k in errs)
    print(f'X1 arm={arm} fault={fault} histories={n} refused={ref} refused_while_C_abandoned={refab} checker_red={red}')
print('# F1')
c=collections.Counter()
for p in glob.glob(f'{D}/out/f1-*-[ab].log'):
  for l in open(p):
    if 'OPUSF1' not in l: continue
    l=re.sub(r'\((\d+), (\d+)\)',r'(\1,\2)',l[l.index('OPUSF1'):])
    f=dict(re.findall(r'(\w+)=(\S+)',l))
    if 'mount' in f: c[(f['arm'],'mount-refused',f['var'],'n')]+=1; continue
    form1='form1' if f['selected']!=f['r'] and f['r_kind_at_mount']=='ValidRoot' else 'other'
    k=(f['arm'],form1,f['var'])
    c[k+('n',)]+=1
    c[k+('refused',)]+=bool(re.search(r'(unmount|w\d+)=Err',l))
    c[k+('red',)]+=('checker=-' not in l)
    c[k+('r_overwritten',)]+=(f['r_units_overwritten']!='0')
keys=sorted({k[:3] for k in c})
for k in keys:
  print('F1 arm=%s %s var=%s histories=%d refused=%d checker_red=%d r_units_overwritten=%d'%(k+(c[k+('n',)],c[k+('refused',)],c[k+('red',)],c[k+('r_overwritten',)])))
