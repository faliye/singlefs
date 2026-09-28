import sys, re, collections
sys.argv = sys.argv
exec(open('/tmp/claude-1000/abandoned-floor-r1/opus/tools/compare.py').read().split("paths = sys.argv[1:]")[0])
a = load(sys.argv[1]); b = load(sys.argv[2])
groups = collections.Counter()
for k in sorted(a):
    if k in b and a[k][2:] != b[k][2:]:
        gap = 'gap' if 'all_abandoned=true' in k[1] else ('nogap' )
        change = f"{a[k][2]}/{a[k][5]} -> {b[k][2]}/{b[k][5]}"
        groups[(k[0], gap, change)] += 1
for g, n in sorted(groups.items()):
    print(n, g)
