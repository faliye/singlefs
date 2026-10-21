"""把一份 run 产物的 name=sig 行按（段、择到哪个、恢复、只读挂载、可写挂载、checker 红、判定）合并计数。用法：python3 summarize.py <产物>"""
import sys,re,collections
agg=collections.Counter()
for line in open(sys.argv[1]):
    if not line.startswith("name=sig"): continue
    count=int(re.search(r"count=(\d+)",line).group(1))
    phase=re.search(r"phase=(\S+)",line).group(1)
    choice=re.search(r"choice=(\S+)",line).group(1)
    verdict=line.split("verdict=",1)[1].strip()
    w=re.search(r"writable=(\S+)",line).group(1)
    ro=re.search(r"read_only=(\S+)",line).group(1)
    rec=re.search(r"recover=(\S+)",line).group(1)
    red=re.search(r"raw_red=(\[[^\]]*\])",line).group(1)
    agg[(phase,choice,rec,ro,w,red,verdict)]+=count
for k,v in sorted(agg.items()): print(v,*k)
