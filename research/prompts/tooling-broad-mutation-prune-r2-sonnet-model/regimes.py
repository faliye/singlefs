# 两种读法下 14 个提交各要重判多少条：L1 = 第二版字面（全局项含 litmus 与 stage-inputs.tsv；常量闭包按文件，format 的 lib.rs 一变，直接引用它的 25 个文件 4940 条都进）；
# L2 = 松一档（litmus 与 stage-inputs.tsv 并进读它的那几条测试而不是全局；常量闭包按常量项）。
# 两边都先取「被改函数 + 函数外面的部分按文件（F2）」的有效条数 eff（commits.json）。L2 的 format 项按 max(eff, 常量项命中) 算，是下界（并集更大）。
import json,re,subprocess
cs={x['c']:x for x in json.load(open('commits.json')) if x['c']!='53c9f384'}
item={}
for l in open('out/fmtitem.txt'):
    m=re.match(r'(\w{8}) format consts changed .*mutants (\d+) ',l)
    if m: item[m.group(1)]=int(m.group(2))
glob={}
for l in open('out/globals.txt'):
    m=re.match(r'(\w{8}):(.*)$',l)
    if m: glob[m.group(1)]=m.group(2).split()
mix_full=(241*72.4+39*0.9)/280;mix_T=(241*22.8+39*0.9)/280
rows=[]
for c,x in cs.items():
    g=glob.get(c,[])
    base=[p for p in g if p in('Cargo.lock','Cargo.toml','.cargo') or p.endswith('Cargo.toml')]
    data=[p for p in g if p=='litmus' or p.endswith('stage-inputs.tsv')]
    fmt='format/lib.rs' in g
    L1=5091 if (base or data) else max(x['eff'],4940 if fmt else 0)
    L2=5091 if base else max(x['eff'],item.get(c,0) if fmt else 0)
    rows.append((c,round(x['eff']),L1,L2))
rows.sort(key=lambda r:r[2])
import statistics
for c,e,a,b in rows: print(c,'eff',e,'L1',a,'L2',b,'hours full L1/L2 %.1f/%.1f  T-first %.1f/%.1f'%(a*mix_full/3600,b*mix_full/3600,a*mix_T/3600,b*mix_T/3600))
for lab,i in (('eff',1),('L1',2),('L2',3)):
    v=[r[i] for r in rows];print(lab,'median %d (%.1f%%) -> %.1f h full / %.1f h T-first'%(statistics.median(v),100*statistics.median(v)/5091,statistics.median(v)*mix_full/3600,statistics.median(v)*mix_T/3600))
print('commits at 100%% of the list: L1 %d of %d, L2 %d of %d'%(sum(1 for r in rows if r[2]>=5091),len(rows),sum(1 for r in rows if r[3]>=5091),len(rows)))
