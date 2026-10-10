# 用法：python3 iso.py '<b3 日志通配>'：同一段历史上丙（bing）与对照臂 exact（R 的真账）收尾时隔离槽数之差。
import sys,re,glob,collections
by=collections.defaultdict(dict)
for f in glob.glob(sys.argv[1]):
    for line in open(f):
        if 'OPUS10ZGHI arm=' not in line: continue
        g=lambda k: re.search(k+r'=(\S+)',line).group(1)
        iso=g('isolated_at_end')
        v=None if iso=='no-session' else sum(int(x) for x in iso.split('+'))
        by[(g('r'),g('actions'))][g('arm')]=v
diff=collections.Counter(); more=collections.Counter(); mx=collections.Counter()
for (r,a),d in by.items():
    if d.get('bing') is None or d.get('exact') is None: continue
    dlt=d['bing']-d['exact']
    diff[r]+=1
    if dlt>0: more[r]+=1
    mx[r]=max(mx[r],dlt)
for r in sorted(diff): print(f"r={r} paired_histories_with_session={diff[r]} bing_isolates_more={more[r]} max_extra_isolated_slots={mx[r]}")
