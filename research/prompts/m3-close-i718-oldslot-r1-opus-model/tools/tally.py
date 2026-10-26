import sys,collections
C=['today_or_bing','jia','yi_literal','yi_after_first_lap','jia_any_lap_of_next_slot','jia_any_unreadable_slot','ding_jia_with_next_record','ding_yi_with_next_record']
rows=[l.split() for l in open(sys.argv[1])]
tally=collections.Counter(); bases=collections.defaultdict(set)
for r in rows:
    kv=dict(x.split('=',1) for x in r[1:] if '=' in x and not x.startswith('newest'))
    key=(kv['cell'],kv['image']); bases[key].add(kv['base_last'])
    for c in C:
        if c in kv: tally[(key,c,kv[c])]+=1
print('格\t镜像\t段数\t'+'\t'.join(C))
for key in sorted(bases):
    cells=[]
    for c in C:
        d={v:n for (k,cc,v),n in tally.items() if k==key and cc==c}
        cells.append(' '.join(f'{v}:{n}' for v,n in sorted(d.items())))
    print(f'{key[0]}\t{key[1]}\t{len(bases[key])}\t'+'\t'.join(cells))
